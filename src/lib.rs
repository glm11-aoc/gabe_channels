mod application;
pub mod channel;
mod device;
mod network;

use crate::device::DeviceChannel;
use application::ApplicationChannel;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub enum Channel<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    #[cfg(feature = "application")]
    Application(Arc<ApplicationChannel<T>>),
    #[cfg(feature = "device")]
    Device(Arc<DeviceChannel<T>>),
    #[cfg(feature = "network")]
    Network(Arc<ApplicationChannel<T>>),
}

pub enum ChannelType {
    #[cfg(feature = "application")]
    Application,
    #[cfg(feature = "device")]
    Device,
    #[cfg(feature = "network")]
    Network,
}

#[derive(Debug)]
pub enum ChannelErrors {
    NoneAvailable,
    Poisoned,
    Full,
    Closed,
}

trait Closeable {
    fn close(&self) -> Result<(), ChannelErrors>;
}

trait RChannel<T> {
    fn read(&self) -> Result<T, ChannelErrors>;
    fn try_read(&self) -> Result<T, ChannelErrors>;
}

trait WChannel<T> {
    fn write(&self, val: T) -> Result<(), ChannelErrors>;
    fn try_write(&self, val: T) -> Result<(), ChannelErrors>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Channel;
    use std::{
        sync::{Arc, Mutex},
        thread,
    };

    #[test]
    fn basic_test() {
        let queue = Channel::<i32>::new(ChannelType::Application, 20).unwrap();
        let mut result = 0;
        println!("1");
        let producer_queue = queue.clone();
        producer_queue.write(1).expect("Failed to send");
        println!("Produced: {}", 1);
        println!("2");
        let consumer_queue = queue.clone();
        *&mut result = consumer_queue.read().expect("Failed to receive");
        println!("3");
        assert_eq!(result, 1)
    }
    #[test]
    fn threaded_test() {
        // if this is too big, it'll stack overflow unless you use --release (thanks to storing the result on stack before moving it) :-(
        const SIZE: usize = 100;
        let queue = Channel::<usize>::new(ChannelType::Application, 5).unwrap();
        let result = Arc::new(Mutex::new(vec![0; SIZE]));
        let res_clone = result.clone();
        let consumer_queue = queue.clone();
        let consumer_thread = thread::spawn(move || {
            let mut i = 0;
            while i < SIZE {
                let mut res = res_clone.lock().unwrap();
                res[i as usize] = consumer_queue.read().expect("Failed to receive");
                i += 1;
            }
        });
        let producer_queue = queue.clone();
        let producer_thread = thread::spawn(move || {
            let mut i = 0;
            while i < SIZE {
                producer_queue.write(i).expect("Failed to send");
                i += 1;
            }
        });
        producer_thread.join().unwrap();
        consumer_thread.join().unwrap();
        let mut i = 0;
        while i < SIZE {
            assert_eq!(result.lock().unwrap()[i], i);
            i += 1;
        }
    }
}

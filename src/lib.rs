mod application;
pub mod channel;
mod device;
pub mod monodirectional;
mod network;

use application::ApplicationChannel;

pub enum Channel<T: Clone + Send + Sync> {
    #[cfg(feature = "application")]
    Application(*mut ApplicationChannel<T>),
    #[cfg(feature = "device")]
    Device(*mut ApplicationChannel<T>),
    #[cfg(feature = "network")]
    Network(*mut ApplicationChannel<T>),
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
    Full,
    Closed,
}

trait Counted {
    fn increment_counter(&self);
    fn decrement_counter(&self);
    fn references(&self) -> usize;
}

trait Closeable {
    fn close(&mut self);
}

trait RChannel<T> {
    fn read(&mut self) -> Result<T, ChannelErrors>;
    fn try_read(&mut self) -> Result<T, ChannelErrors>;
}

trait WChannel<T> {
    fn write(&mut self, val: T) -> Result<(), ChannelErrors>;
    fn try_write(&mut self, val: T) -> Result<(), ChannelErrors>;
}

/*
pub struct Channel<T: Clone + Send + Sync> {
    obj: ChannelType<T>,
}
*/

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
        let queue = Channel::<i32>::new(ChannelType::Application, 20);
        let mut result = 0;
        println!("1");
        let mut producer_queue = queue.clone();
        producer_queue.write(1).expect("Failed to send");
        println!("Produced: {}", 1);
        println!("2");
        let mut consumer_queue = queue.clone();
        *&mut result = consumer_queue.read().expect("Failed to receive");
        println!("3");
        assert_eq!(result, 1)
    }
    #[test]
    fn threaded_test() {
        // if this is too big, it'll stack overflow unless you use --release (thanks to storing the result) :-(
        const SIZE: usize = 10000;
        let queue = Channel::<usize>::new(ChannelType::Application, 5);
        let result = Arc::new(Mutex::new(vec![0; SIZE]));
        let res_clone = result.clone();
        let mut consumer_queue = queue.clone();
        let consumer_thread = thread::spawn(move || {
            let mut i = 0;
            while i < SIZE {
                let mut res = res_clone.lock().unwrap();
                res[i as usize] = consumer_queue.read().expect("Failed to receive");
                i += 1;
            }
        });
        let mut producer_queue = queue.clone();
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
            assert!(result.lock().unwrap()[i] == i);
            i += 1;
        }
    }
}

use crate::{
    application::ApplicationChannel, Channel, ChannelErrors, ChannelType, Closeable, Counted,
    RChannel, WChannel,
};
use std::mem::forget;

unsafe impl<T: Clone + Send + Sync> Send for Channel<T> {}

impl<T: Clone + Send + Sync + 'static> Clone for Channel<T> {
    fn clone(&self) -> Self {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.increment_counter();
        return match self {
            Channel::Application(v) => Channel::Application(*v),
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
    }
}

impl<T: Clone + Send + Sync> Drop for Channel<T> {
    fn drop(&mut self) {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.decrement_counter();
        if obj.references() == 0 {
            drop(unsafe { Box::from_raw(*ptr) })
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Channel<T> {
    pub fn new(variant: ChannelType, buffer_size: usize) -> Self {
        let mut val = match variant {
            ChannelType::Application => Box::new(ApplicationChannel::<T>::new(buffer_size)),
            ChannelType::Device => panic!("Not Implmented"),
            ChannelType::Network => panic!("Not Implmented"),
        };
        let ret = match variant {
            ChannelType::Application => Channel::Application(&mut *val),
            ChannelType::Device => todo!(),
            ChannelType::Network => todo!(),
        };
        forget(val);
        return ret;
    }
    pub fn write(&mut self, val: T) -> Result<(), ChannelErrors> {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.write(val)
    }
    pub fn try_write(&mut self, val: T) -> Result<(), ChannelErrors> {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.try_write(val)
    }
    pub fn read(&mut self) -> Result<T, ChannelErrors> {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.read()
    }
    pub fn try_read(&mut self) -> Result<T, ChannelErrors> {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.try_read()
    }
    pub fn close(&self) {
        let ptr = match self {
            Channel::Application(v) => v,
            Channel::Device(_) => todo!(),
            Channel::Network(_) => todo!(),
        };
        let obj = unsafe { &mut **ptr };
        obj.close();
    }
}

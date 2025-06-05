use std::ops::DerefMut;
use crate::{
    application::ApplicationChannel, Channel, ChannelErrors, ChannelType, Closeable,
    RChannel, WChannel,
};
use std::sync::{Arc, Mutex};
use crate::ChannelType::Application;

unsafe impl<T: Clone + Send + Sync> Send for Channel<T> {}

impl<T: Clone + Send + Sync + 'static> Channel<T> {
    pub fn new(variant: ChannelType, buffer_size: usize) -> Self {
        match variant {
            Application => Channel::Application(Arc::new(ApplicationChannel::<T>::new(buffer_size))),
            ChannelType::Device => panic!("Not Implemented"),
            ChannelType::Network => panic!("Not Implemented"),
        }
    }
    pub fn write(&mut self, val: T) -> Result<(), ChannelErrors> {
        match self{
            Channel::Application(c) => c.write(val),
            Channel::Device(c) => todo!(),
            Channel::Network(c) => todo!()
        }
    }
    pub fn try_write(&mut self, val: T) -> Result<(), ChannelErrors> {
        match self{
            Channel::Application(c) => c.try_write(val),
            Channel::Device(c) => todo!(),
            Channel::Network(c) => todo!()
        }
    }
    pub fn read(&mut self) -> Result<T, ChannelErrors> {
        match self{
            Channel::Application(c) => c.read(),
            Channel::Device(c) => todo!(),
            Channel::Network(c) => todo!()
        }
    }
    pub fn try_read(&mut self) -> Result<T, ChannelErrors> {
        match self{
            Channel::Application(c) => c.try_read(),
            Channel::Device(c) => todo!(),
            Channel::Network(c) => todo!()
        }
    }
    pub fn close(&self) {
        match self{
            Channel::Application(c) => c.close(),
            Channel::Device(c) => todo!(),
            Channel::Network(c) => todo!()
        }
    }
}

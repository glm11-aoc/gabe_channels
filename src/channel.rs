use crate::{
    application::ApplicationChannel, Channel, ChannelErrors, ChannelType, Closeable,
    RChannel, WChannel,
};
use std::sync::Arc;

unsafe impl<T: Clone + Send + Sync> Send for Channel<T> {}

impl<T: Clone + Send + Sync + 'static> Channel<T> {
    pub fn new(variant: ChannelType, buffer_size: usize) -> Self {
        match variant {
            #[cfg(feature = "application")]
            ChannelType::Application => Channel::Application(Arc::new(ApplicationChannel::<T>::new(buffer_size))),
            #[cfg(feature = "device")]
            ChannelType::Device => todo!("Not Implemented"),
            #[cfg(feature = "network")]
            ChannelType::Network => todo!("Not Implemented"),
        }
    }
    pub fn write(&self, val: T) -> Result<(), ChannelErrors> {
        match self{
            #[cfg(feature = "application")]
            Channel::Application(c) => c.write(val),
            #[cfg(feature = "device")]
            Channel::Device(c) => c.write(val),
            #[cfg(feature = "network")]
            Channel::Network(c) => c.write(val)
        }
    }
    pub fn try_write(&self, val: T) -> Result<(), ChannelErrors> {
        match self{
            #[cfg(feature = "application")]
            Channel::Application(c) => c.try_write(val),
            #[cfg(feature = "device")]
            Channel::Device(c) => c.try_write(val),
            #[cfg(feature = "network")]
            Channel::Network(c) => c.try_write(val)
        }
    }
    pub fn read(&self) -> Result<T, ChannelErrors> {
        match self{
            #[cfg(feature = "application")]
            Channel::Application(c) => c.read(),
            #[cfg(feature = "device")]
            Channel::Device(c) => c.read(),
            #[cfg(feature = "network")]
            Channel::Network(c) => c.read()
        }
    }
    pub fn try_read(&self) -> Result<T, ChannelErrors> {
        match self{
            #[cfg(feature = "application")]
            Channel::Application(c) => c.try_read(),
            #[cfg(feature = "device")]
            Channel::Device(c) => c.try_read(),
            #[cfg(feature = "network")]
            Channel::Network(c) => c.try_read()
        }
    }
    pub fn close(&self) -> Result<(), ChannelErrors> {
        match self{
            #[cfg(feature = "application")]
            Channel::Application(c) => c.close(),
            #[cfg(feature = "device")]
            Channel::Device(c) => c.close(),
            #[cfg(feature = "network")]
            Channel::Network(c) => c.close()
        }
    }
}

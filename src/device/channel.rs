use crate::{
    device::{client, server},
    ChannelErrors, Closeable, RChannel, WChannel,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Condvar, Mutex, MutexGuard};

enum ChannelClass<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    Server(server::LocalDeviceStack<T>),
    Client(client::RemoteDeviceStack<T>),
}
pub struct DeviceChannel<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    available: Mutex<ChannelClass<T>>,
    read_cvar: Condvar,
    write_cvar: Condvar,
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> DeviceChannel<T> {
    pub fn new(buffer_size: usize) -> Result<Self, ChannelErrors> {
        todo!()
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> RChannel<T> for DeviceChannel<T> {
    fn read(&self) -> Result<T, ChannelErrors> {
        todo!()
    }

    fn try_read(&self) -> Result<T, ChannelErrors> {
        todo!()
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> WChannel<T> for DeviceChannel<T> {
    fn write(&self, val: T) -> Result<(), ChannelErrors> {
        todo!()
    }

    fn try_write(&self, val: T) -> Result<(), ChannelErrors> {
        todo!()
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> Closeable for DeviceChannel<T> {
    fn close(&self) -> Result<(), ChannelErrors> {
        todo!()
    }
}

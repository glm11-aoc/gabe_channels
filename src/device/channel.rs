use crate::{
    device::{client::RemoteDeviceStack, server::LocalDeviceStack},
    ChannelErrors, ChannelType, Closeable, CreationErrors, DeviceConfig, RChannel, WChannel,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{fs, sync::{Condvar, Mutex, MutexGuard}};

enum ChannelClass<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    Server(LocalDeviceStack<T>),
    Client(RemoteDeviceStack<T>),
}
pub struct DeviceChannel<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    available: Mutex<ChannelClass<T>>,
    read_cvar: Condvar,
    write_cvar: Condvar,
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> DeviceChannel<T> {
    pub fn new(config: DeviceConfig, buffer_size: usize) -> Result<Self, CreationErrors> {
        // check this config.path, then if it exists try to create a client
        match fs::exists(&config.path) {
            Ok(exists) => {
                match exists {
                    true => Ok(DeviceChannel {
                        available: Mutex::new(ChannelClass::Client(RemoteDeviceStack::new(config.path))),
                        read_cvar: Condvar::new(),
                        write_cvar: Condvar::new()
                    }),
                    false => Ok(DeviceChannel {
                        available: Mutex::new(ChannelClass::Server(LocalDeviceStack::new(config.path))),
                        read_cvar: Condvar::new(),
                        write_cvar: Condvar::new()
                    })
                }
            },
            Err(_) => todo!("Handle fs errors ;(")
        }
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

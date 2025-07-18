use crate::device::DeviceChannel;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

pub struct RemoteDeviceStack<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    _phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> RemoteDeviceStack<T> {
    pub fn new(path: String) -> RemoteDeviceStack<T> {
        todo!()
    }
}

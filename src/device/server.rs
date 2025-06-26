use crate::application::ApplicationChannel;
use crate::device::DeviceChannel;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct LocalDeviceStack<T: Serialize + DeserializeOwned + Clone + Send + Sync> {
    application_channel: ApplicationChannel<T>,
}

impl<T: Serialize + DeserializeOwned + Clone + Send + Sync> LocalDeviceStack<T> {
    pub fn new() -> DeviceChannel<T> {
        todo!()
    }
}

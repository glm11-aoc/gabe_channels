/* Not gonna use this in channels lib
use crate::{Counted, ChannelErrors, ChannelType};

trait MonoSendChannel<T>: Counted {
    fn write(&mut self, val: T) -> Result<(), ChannelErrors>;
    fn host(&mut self) -> ChannelErrors;
    fn close(&mut self);
}

trait MonoReceiveChannel<T>: Counted {
    fn read(&mut self) -> Result<T, ChannelErrors>;
    fn connect(&mut self) -> Result<T, ChannelErrors>;
    fn close(&mut self);
}

pub struct MonoDirectionalChannelSender<T> {
    obj: *mut dyn MonoSendChannel<T>,
}

pub struct MonoDirectionalChannelReceiver<T> {
    obj: *mut dyn MonoReceiveChannel<T>,
}

impl<T: Clone + Send + Sync> MonoDirectionalChannelSender<T> {
    // add new method :-)
    pub fn new(variant: ChannelType) -> Self {
        todo!()
    }
    pub fn host(&mut self) {}

    pub fn write(&mut self, val: T) -> Result<(), ChannelErrors> {
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        obj.write(val)
    }
    pub fn close(&mut self) {
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        obj.close();
    }
}

impl<T: Clone + Send + Sync> MonoDirectionalChannelReceiver<T> {
    pub fn new(variant: ChannelType) -> Self {
        todo!()
    }
    pub fn connect(&mut self) -> Option<ChannelErrors> {
        None
    }
    pub fn read(&mut self) -> Result<T, ChannelErrors> {
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        obj.read()
    }
    pub fn close(&mut self) {
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        obj.close();
    }
}

*/

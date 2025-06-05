use crate::{ChannelErrors, Closeable, RChannel, WChannel};
use std::mem::replace;
use std::sync::{
    Condvar, Mutex,
};

pub struct ApplicationChannel<T: Clone + Send + Sync> {
    available: Mutex<ApplicationStack<T>>,
    read_cvar: Condvar,
    write_cvar: Condvar,
}

struct ApplicationStack<T: Clone + Send + Sync> {
    buffer: Box<[Option<T>]>,
    length: usize,
    read_offset: usize,
    write_offset: usize,
    open: bool,
}

impl<T: Clone + Send + Sync> WChannel<T> for ApplicationChannel<T> {
    fn write(&self, item: T) -> Result<(), ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let mut stack = lock.lock().unwrap();
        while stack.read_offset == (stack.write_offset + 1) % stack.length && stack.open {
            stack = self.write_cvar.wait(stack).unwrap();
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }
        // Write logic
        let offset = stack.write_offset;
        stack.buffer[offset] = Some(item);
        stack.write_offset = (stack.write_offset + 1) % stack.length;
        self.read_cvar.notify_one();
        Ok(())
    }

    fn try_write(&self, item: T) -> Result<(), ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let mut stack = lock.lock().unwrap();
        if stack.read_offset == (stack.write_offset + 1) % stack.length && stack.open {
            return Err(ChannelErrors::NoneAvailable);
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }
        // Write logic
        let offset = stack.write_offset;
        stack.buffer[offset] = Some(item);
        stack.write_offset = (stack.write_offset + 1) % stack.length;
        self.read_cvar.notify_one();
        Ok(())
    }
}

impl<T: Clone + Send + Sync> RChannel<T> for ApplicationChannel<T> {
    fn read(&self) -> Result<T, ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let mut stack = lock.lock().unwrap();
        while stack.read_offset == stack.write_offset && stack.open {
            stack = self.read_cvar.wait(stack).unwrap();
        }

        // Channel is closed
        if !stack.open {
            return Result::Err(ChannelErrors::Closed);
        }

        // Retrieval logic
        let offset = stack.read_offset;
        match &mut stack.buffer[offset] {
            Some(_) => {
                let res = Result::Ok(replace(&mut stack.buffer[offset], None).unwrap());
                stack.read_offset = (stack.read_offset + 1) % stack.length;
                self.write_cvar.notify_one();
                res
            }
            None => panic!("No value found in queue during synchronous call"),
        }
    }

    fn try_read(&self) -> Result<T, ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let mut stack = lock.lock().unwrap();
        while stack.read_offset == stack.write_offset && stack.open {
            return Err(ChannelErrors::NoneAvailable);
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }

        // Retrieval logic
        let offset = stack.read_offset;
        match &stack.buffer[offset] {
            Some(_) => {
                let res = Result::Ok(replace(&mut stack.buffer[offset], None).unwrap());
                stack.read_offset = (stack.read_offset + 1) % stack.length;
                self.write_cvar.notify_one();
                res
            }
            None => panic!("No value found in queue during synchronous call"),
        }
    }
}

impl<T: Clone + Send + Sync> Closeable for ApplicationChannel<T> {
    fn close(&self) {
        let lock = &self.available;
        let mut stack = lock.lock().unwrap();

        // set to closed
        stack.open = false;

        // clear data
        for i in 0..stack.length {
            stack.buffer[i] = None;
        }
        // notify all queues that this queue is closed
        self.read_cvar.notify_all();
        self.write_cvar.notify_all();
    }
}

impl<T: Clone + Send + Sync> ApplicationChannel<T> {
    pub fn new(length: usize) -> ApplicationChannel<T> {
        if length >= core::usize::MAX {
            panic!(
                "Doesn't support values greater than usize-1 to ensure reliable overflow handling"
            )
        }
        // getting memory for stack
        let stack = vec![None; length].into_boxed_slice();
        ApplicationChannel {
            available: Mutex::new(ApplicationStack{
                buffer: stack,
                length,
                read_offset: 0,
                write_offset: 0,
                open: true,
            }),
            read_cvar: Condvar::new(),
            write_cvar: Condvar::new(),
        }
    }
}

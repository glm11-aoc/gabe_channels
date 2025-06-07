use crate::{ChannelErrors, Closeable, RChannel, WChannel};
use std::mem::replace;
use std::sync::{Condvar, Mutex, MutexGuard};

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
        let mut stack = self.acquire_stack_lock()?;
        while stack.read_offset == (stack.write_offset + 1) % stack.length && stack.open {
            stack = match self.write_cvar.wait(stack) {
                Ok(v) => v,
                Err(_) => return Err(ChannelErrors::Poisoned)
            };
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }
        // Write logic
        self.write(&mut stack, item);
        Ok(())
    }

    fn try_write(&self, item: T) -> Result<(), ChannelErrors> {
        let mut stack = self.acquire_stack_lock()?;
        // Waiting logic
        if stack.read_offset == (stack.write_offset + 1) % stack.length && stack.open {
            return Err(ChannelErrors::NoneAvailable);
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }
        // Write logic
        self.write(&mut stack, item);
        Ok(())
    }
}

impl<T: Clone + Send + Sync> RChannel<T> for ApplicationChannel<T> {
    fn read(&self) -> Result<T, ChannelErrors> {
        let mut stack = self.acquire_stack_lock()?;
        // Waiting logic
        while stack.read_offset == stack.write_offset && stack.open {
            stack = match self.read_cvar.wait(stack) {
                Ok(v) => v,
                Err(_) => return Err(ChannelErrors::Poisoned)
            };
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }

        // Retrieval logic
        let offset = stack.read_offset;
        self.read(&mut stack, offset)
    }

    fn try_read(&self) -> Result<T, ChannelErrors> {
        let mut stack = self.acquire_stack_lock()?;
        // Waiting logic
        while stack.read_offset == stack.write_offset && stack.open {
            return Err(ChannelErrors::NoneAvailable);
        }

        // Channel is closed
        if !stack.open {
            return Err(ChannelErrors::Closed);
        }

        // Retrieval logic
        let offset = stack.read_offset;
        self.read(&mut stack, offset)
    }
}

impl<T: Clone + Send + Sync> Closeable for ApplicationChannel<T> {
    fn close(&self) -> Result<(), ChannelErrors> {
        let mut stack = self.acquire_stack_lock()?;

        // set to closed
        stack.open = false;

        // clear data
        for i in 0..stack.length {
            stack.buffer[i] = None;
        }
        // notify all queues that this queue is closed
        self.read_cvar.notify_all();
        self.write_cvar.notify_all();
        Ok(())
    }
}

impl<T: Clone + Send + Sync> ApplicationChannel<T> {
    pub fn new(length: usize) -> ApplicationChannel<T> {
        if length >= usize::MAX {
            panic!(
                "Doesn't support values greater than usize-1 to ensure reliable overflow handling"
            )
        }
        // getting memory for stack
        let stack = vec![None; length].into_boxed_slice();
        ApplicationChannel {
            available: Mutex::new(ApplicationStack {
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

    fn acquire_stack_lock(&self) -> Result<MutexGuard<ApplicationStack<T>>, ChannelErrors> {
        let lock = &self.available;
        match lock.lock() {
            Ok(v) => Ok(v),
            Err(_) => Err(ChannelErrors::Poisoned)
        }
    }

    fn read(&self, stack: &mut MutexGuard<ApplicationStack<T>>, offset: usize) -> Result<T, ChannelErrors> {
        let res = match replace(&mut stack.buffer[offset], None)
        {
            Some(v) => v,
            None => return Err(ChannelErrors::NoneAvailable),
        };
        stack.read_offset = (stack.read_offset + 1) % stack.length;
        self.write_cvar.notify_one();
        Ok(res)
    }

fn write(&self, stack: &mut MutexGuard<ApplicationStack<T>>, val: T) {
    let offset = stack.write_offset;
    stack.buffer[offset] = Some(val);
    stack.write_offset = (stack.write_offset + 1) % stack.length;
    self.read_cvar.notify_one();
}
}

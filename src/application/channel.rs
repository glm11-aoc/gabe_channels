use crate::{ChannelErrors, Closeable, Counted, RChannel, WChannel};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Condvar, Mutex,
};
use std::mem::replace;

pub struct ApplicationChannel<T: Clone + Send + Sync> {
    reference_counter: AtomicUsize,
    stack: Box<[Option<T>]>,
    length: usize,
    read_offset: usize,
    write_offset: usize,
    available: Mutex<()>,
    read_cvar: Condvar,
    write_cvar: Condvar,
    open: bool,
}

impl<T: Clone + Send + Sync> Counted for ApplicationChannel<T> {
    fn increment_counter(&self) {
        self.reference_counter.fetch_add(1, Ordering::SeqCst);
    }
    fn decrement_counter(&self) {
        self.reference_counter.fetch_sub(1, Ordering::SeqCst);
    }
    fn references(&self) -> usize {
        self.reference_counter.load(Ordering::SeqCst)
    }
}

impl<T: Clone + Send + Sync> WChannel<T> for ApplicationChannel<T> {
    fn write(&mut self, item: T) -> Result<(), ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let mut available = lock.lock().unwrap();
        while self.read_offset == self.update_offset(self.write_offset) && self.open {
            available = self.write_cvar.wait(available).unwrap();
        }

        // Channel is closed
        if !self.open {
            return Err(ChannelErrors::Closed);
        }
        // Write logic
        self.stack[self.write_offset] = Some(item);
        self.write_offset = self.update_offset(self.write_offset);

        self.read_cvar.notify_one();
        Ok(())
    }

    fn try_write(&mut self, item: T) -> Result<(), ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let _lock = lock.lock().unwrap();
        if self.read_offset == self.update_offset(self.write_offset) && self.open {
            return Err(ChannelErrors::NoneAvailable);
        }

        // Channel is closed
        if !self.open {
            return Err(ChannelErrors::Closed);
        }
        // Write logic
        self.stack[self.write_offset] = Some(item);
        self.write_offset = self.update_offset(self.write_offset);

        self.read_cvar.notify_one();
        Ok(())
    }
}

impl<T: Clone + Send + Sync> RChannel<T> for ApplicationChannel<T> {
    fn read(&mut self) -> Result<T, ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let mut available = lock.lock().unwrap();
        while self.read_offset == self.write_offset && self.open {
            available = self.read_cvar.wait(available).unwrap();
        }

        // Channel is closed
        if !self.open {
            return Result::Err(ChannelErrors::Closed);
        }

        // Retrieval logic
        match &mut self.stack[self.read_offset] {
            Some(_) => {
                let res = Result::Ok(replace(&mut self.stack[self.read_offset], None).unwrap());
                self.write_cvar.notify_one();
                self.read_offset = self.update_offset(self.read_offset);
                res
            }
            None => panic!("No value found in queue during syncronous call"),
        }
    }

    fn try_read(&mut self) -> Result<T, ChannelErrors> {
        // Waiting logic
        let lock = &self.available;
        let _lock = lock.lock().unwrap();
        while self.read_offset == self.write_offset && self.open {
            return Err(ChannelErrors::NoneAvailable);
        }

        // Channel is closed
        if !self.open {
            return Err(ChannelErrors::Closed);
        }

        // Retrieval logic
        match &self.stack[self.read_offset] {
            Some(a) => {
                self.read_offset = self.update_offset(self.read_offset);
                self.write_cvar.notify_one();
                Result::Ok(a.clone())
            }
            None => panic!("No value found in queue during syncronous call"),
        }
    }
}

impl<T: Clone + Send + Sync> Closeable for ApplicationChannel<T> {
    fn close(&mut self) {
        // set to closed
        self.open = false;
        // clear data
        for i in 0..self.length {
            self.stack[i] = None;
        }
        let lock = &self.available;
        // notify all queues that this queue is closed
        let _guard = lock.lock().unwrap();
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
            reference_counter: 1.into(),
            stack,
            length,
            available: Mutex::new(()),
            read_cvar: Condvar::new(),
            write_cvar: Condvar::new(),
            read_offset: 0,
            write_offset: 0,
            open: true,
        }
    }

    fn update_offset(&self, offset: usize) -> usize {
        let mut new_offset = offset + 1;
        if new_offset == self.length {
            new_offset = 0;
        }
        new_offset
    }
}

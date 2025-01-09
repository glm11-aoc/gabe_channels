use std::{
    mem::forget,
    sync::{atomic::AtomicU64, Condvar, Mutex},
};

struct QueueInternal<T: Clone + Send + Sync> {
    reference_counter: AtomicU64,
    stack: Box<[Option<T>]>,
    length: usize,
    read_offset: usize,
    write_offset: usize,
    available_read_write: (Mutex<()>, Condvar, Condvar),
    open: bool,
}

// Intended to allow for inter-thread process communication where data transferred is able to be safely cloned
pub struct Queue<T: Clone + Send + Sync> {
    obj: *mut QueueInternal<T>,
}

impl<T: Clone + Send + Sync> Clone for Queue<T> {
    fn clone(&self) -> Self {
        // Access dance
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        *(obj.reference_counter.get_mut()) += 1;
        return Queue { obj: ptr };
    }
}

impl<T: Clone + Send + Sync> Drop for Queue<T> {
    fn drop(&mut self) {
        // Access dance
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        if *(obj.reference_counter.get_mut()) == 1 {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        } else {
            *(obj.reference_counter.get_mut()) -= 1
        }
    }
}

unsafe impl<T: Clone + Send + Sync> Send for Queue<T> {}

#[derive(Debug)]
pub enum QueueErrors {
    NoneAvailable,
    Closed,
}

impl<T: Clone + Send + Sync> Queue<T> {
    pub fn new(length: usize) -> Queue<T> {
        if length >= core::usize::MAX {
            panic!(
                "Doesn't support values greater than usize-1 to ensure reliable overflow handling"
            )
        }
        let stack = vec![None; length].into_boxed_slice();
        let mut internal = Box::new(QueueInternal {
            reference_counter: 1.into(),
            stack,
            length,
            available_read_write: (Mutex::new(()), Condvar::new(), Condvar::new()),
            read_offset: 0,
            write_offset: 0,
            open: true,
        });
        let external = Queue {
            obj: &mut *internal,
        };
        // just gotta leak this real quick :-)
        forget(internal);
        return external;
    }

    pub fn write(&mut self, item: T) -> Result<(), QueueErrors> {
        // Access dance
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };

        // Waiting logic
        let (lock, read_cvar, write_cvar) = &obj.available_read_write;
        let mut available = lock.lock().unwrap();
        while obj.read_offset == self.update_offset(obj.write_offset) && obj.open {
            available = write_cvar.wait(available).unwrap();
        }

        // Channel is closed
        if !obj.open {
            return Err(QueueErrors::Closed);
        }
        // Write logic
        obj.stack[obj.write_offset] = Some(item);
        obj.write_offset = self.update_offset(obj.write_offset);

        read_cvar.notify_one();
        Ok(())
    }

    pub fn read(&mut self) -> Result<T, QueueErrors> {
        // Access dance
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };

        // Waiting logic
        let (lock, read_cvar, write_cvar) = &obj.available_read_write;
        let mut available = lock.lock().unwrap();
        while obj.read_offset == obj.write_offset && obj.open {
            available = read_cvar.wait(available).unwrap();
        }

        // Channel is closed
        if !obj.open {
            return Result::Err(QueueErrors::Closed);
        }

        // Retrieval logic
        match &obj.stack[obj.read_offset] {
            Some(a) => {
                obj.read_offset = self.update_offset(obj.read_offset);
                /* needs to notify all threads since I don't differentiate
                the read and write mutex (probably should) */
                write_cvar.notify_one();
                Result::Ok(a.clone())
            }
            None => panic!("No value found in queue during syncronous call"),
        }
    }

    pub fn close(&mut self) {
        // Access dance
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        // set to closed
        obj.open = false;
        // clear data
        for i in 0..obj.length {
            obj.stack[i] = None
        }
        let (lock, read_cvar, write_cvar) = &obj.available_read_write;
        // notify all queues that this queue is closed
        let _guard = lock.lock().unwrap();
        read_cvar.notify_all();
        write_cvar.notify_all();
    }
    fn update_offset(&self, offset: usize) -> usize {
        // Access dance
        let ptr = self.obj;
        let obj = unsafe { &mut *ptr };
        // generate new offset (handle wrapping)
        let mut new_offset = offset + 1;
        if new_offset == obj.length {
            new_offset = 0;
        }
        new_offset
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::*;

    #[test]
    fn basic_test() {
        let queue = Queue::<i32>::new(20);
        let mut result = 0;
        println!("1");
        let mut producer_queue = queue.clone();
        producer_queue.write(1).expect("Failed to send");
        println!("Produced: {}", 1);
        println!("2");
        let mut consumer_queue = queue.clone();
        *&mut result = consumer_queue.read().expect("Failed to receive");
        println!("3");
        assert_eq!(result, 1)
    }
    #[test]
    fn threaded_test() {
        // if this is too big, it'll stack overflow unless you use --release :-(
        const SIZE: usize = 10000;
        let queue = Queue::<usize>::new(5);
        let result = Arc::new(Mutex::new(Box::new([0; SIZE])));
        let res_clone = result.clone();
        let mut consumer_queue = queue.clone();
        let consumer_thread = thread::spawn(move || {
            let mut i = 0;
            while i < SIZE {
                let mut res = res_clone.lock().unwrap();
                res[i as usize] = consumer_queue.read().expect("Failed to receive");
                i += 1;
            }
        });
        let mut producer_queue = queue.clone();
        let producer_thread = thread::spawn(move || {
            let mut i = 0;
            while i < SIZE {
                producer_queue.write(i).expect("Failed to send");
                i += 1;
            }
        });
        producer_thread.join().unwrap();
        consumer_thread.join().unwrap();
        let mut i = 0;
        while i < SIZE {
            assert!(result.lock().unwrap()[i] == i);
            i += 1;
        }
    }
}

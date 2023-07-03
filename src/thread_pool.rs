use crossbeam_channel::Sender;
use log::error;
use std::panic::AssertUnwindSafe;
use std::{panic, thread};

use crate::error::Result;

pub trait ThreadPool {
    /// Creates a new thread pool, immediately spawning the specified number of threads.
    ///
    /// Returns an error if any thread fails to spawn. All previously-spawned threads are terminated.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;
    /// Spawn a function into the thread pool.
    ///
    /// Spawning always succeeds, but if the function panics the thread pool continues
    /// to operate with the same number of threads — the thread count is not reduced
    /// nor is the thread pool destroyed, corrupted or invalidated.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(NaiveThreadPool {})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}

pub struct SharedQueueThreadPool {
    queue: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        let (sender, receiver) =
            crossbeam_channel::unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _ in 0..threads {
            let receiver = receiver.clone();
            thread::spawn(move || {
                for job in receiver.iter() {
                    if let Err(error) = panic::catch_unwind(AssertUnwindSafe(job)) {
                        error!("thread worker panicked with: {:?}", error);
                    }
                }
            });
        }
        Ok(SharedQueueThreadPool { queue: sender })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.send(Box::new(job)).unwrap();
    }
}

pub struct RayonThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(RayonThreadPool {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(threads as usize)
                .build()?,
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.spawn(job)
    }
}

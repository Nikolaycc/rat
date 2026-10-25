use std::{
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use crate::error::PoolCreationError;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> io::Result<Self> {
        let thd = thread::Builder::new().name(format!("capture_worker_{id}"));

        let thread = thd.spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                if let Ok(job) = message {
                    println!("Worker {id} got a job; executing.");

                    job();
                } else {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        })?;

        Ok(Worker {
            id,
            thread: Some(thread),
        })
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new `ThreadPool`.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// #Errors
    ///
    /// Will return 'Err' if the size is zero.
    pub fn new(size: usize) -> Result<Self, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::ZeroSize);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver))?);
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Executes a function on a worker thread.
    ///
    /// The function contains the logic that runs in the pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[test]
fn zero_size_pool() {
    let err = ThreadPool::new(0).unwrap_err();

    assert!(matches!(err, PoolCreationError::ZeroSize))
}

#[test]
fn execute_thread() {
    let pool = ThreadPool::new(5).unwrap();

    pool.execute(|| {})
}

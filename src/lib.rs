use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Creates a new [`ThreadPool`].
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// Panics if thread size is equal to 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

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
    }
}

struct Worker {
    id: usize,
    handler: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            handler: Some(thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        println!("Got some job for thread {}", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected.");
                        break;
                    }
                }
            })),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        println!("Dropping worker {}", self.id);
        self.handler.take().unwrap().join().unwrap();
        println!("Worker {} dropped", self.id);
    }
}

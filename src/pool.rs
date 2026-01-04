use std::io::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self};

use crate::crawler::DirectoryCrawl;

type Job = Box<dyn FnOnce() -> Result<DirectoryCrawl, Error> + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    pub fn new(num_workers: usize, res_sender: Sender<Result<DirectoryCrawl, Error>>) -> Self {
        let (tx, rx) = mpsc::channel();
        let mut workers = Vec::with_capacity(num_workers);

        let reciever = Arc::new(Mutex::new(rx));

        for i in 0..num_workers {
            let reciever = Arc::clone(&reciever);
            let worker = Worker::new(i, reciever, res_sender.clone());
            workers.push(worker);
        }

        Self {
            workers,
            sender: Some(tx),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() -> Result<DirectoryCrawl, Error> + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers.drain(..) {
            println!("ThreadPool: Shutting down worker {}", worker.id);
            worker.handle.join().unwrap();
        }
    }
}

pub struct Worker {
    id: usize,
    handle: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(
        id: usize,
        reciever: Arc<Mutex<Receiver<Job>>>,
        res_sender: Sender<Result<DirectoryCrawl, Error>>,
    ) -> Self {
        println!("Creating worker {id}");
        let handle = thread::spawn(move || {
            let sender = res_sender;
            loop {
                let message = reciever.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        // println!("Worker {id} got a job");
                        let res = job();
                        sender.send(res).unwrap();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; Shutting down!");
                        break;
                    }
                }
            }
        });

        Self { id, handle }
    }
}

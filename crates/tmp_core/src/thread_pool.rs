use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub enum Message {
    Job(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// `size` is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if `size` is 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = unbounded();

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool {
            workers,
            sender,
            receiver,
        }
    }

    /// Sends a job to the pool.
    ///
    /// The closure `f` must be `Send` and 'static.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::Job(job)).unwrap();
    }

    /// Waits for all jobs to complete and then terminates the workers.
    pub fn join(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    pub fn get_receiver(&self) -> Receiver<Message> {
        self.receiver.clone()
    }
}

pub fn execute_job_from_queue(receiver: Receiver<Message>) {
    loop {
        let message = receiver.try_recv();

        match message {
            Ok(Message::Job(job)) => {
                job();
            }
            Ok(Message::Terminate) => {
                // ジョブが終了メッセージの場合はループを抜ける
                break;
            }
            Err(TryRecvError::Empty) => {
                // キューが空の場合はループを抜ける
                break;
            }
            Err(_) => {
                // その他のエラー（通常は起こらない）
                break;
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.join();
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Receiver<Message>) -> Worker {
        let thread = thread::spawn(move || loop {
            if let Ok(message) = receiver.recv() {
                match message {
                    Message::Job(job) => {
                        log::debug!("Worker {} got a job; executing.", id);
                        job();
                    }
                    Message::Terminate => {
                        log::debug!("Worker {} was told to terminate.", id);
                        break;
                    }
                }
            } else {
                break;
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(8);

        for i in 0..8 {
            pool.execute(move || {
                println!("Executing job {}, {:?}", i, std::thread::current());
            });
        }

        let receiver = pool.get_receiver();
        execute_job_from_queue(receiver);

        println!("end");
    }
}

use log;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    Job(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
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

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
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

    pub fn get_receiver(&self) -> Arc<Mutex<mpsc::Receiver<Message>>> {
        self.receiver.clone()
    }
}

pub fn execute_job_from_queue(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) {
    loop {
        let message = {
            let lock = receiver.lock().unwrap();
            lock.try_recv()
        };

        match message {
            Ok(Message::Job(job)) => {
                job();
            }
            Ok(Message::Terminate) => {
                // ジョブが終了メッセージの場合はループを抜ける
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

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
        let pool = ThreadPool::new(2);

        for i in 0..4 {
            pool.execute(move || {
                println!("Executing job {}", i);
            });
        }

        // Sleep for a while to allow threads to execute
        thread::sleep(std::time::Duration::from_secs(2));
    }
}

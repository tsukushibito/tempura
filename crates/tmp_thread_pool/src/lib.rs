use std::{sync::Mutex, thread};


struct Task {
    f: Box<dyn FnOnce() + Send>,
}

struct ThreadPool {
    workers: Mutex<Vec<thread::JoinHandle<()>>>,
    tasks: Mutex<Vec<Task>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _t = Task {f: Box::new(|| {})} ;
        println!("test");
    }
}

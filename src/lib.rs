#![feature(async_closure, async_fn_traits)]
#![feature(unboxed_closures)]

use std::ops::AsyncFn;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{spawn, JoinHandle};

#[allow(dead_code)]
pub struct AsyncWorker {
    running: Arc<RwLock<bool>>,
    id: usize,
    handle: tokio::task::JoinHandle<()>,
}

#[allow(dead_code)]
pub struct Worker {
    running: Arc<RwLock<bool>>,
    id: usize,
    handle: JoinHandle<()>,
}

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    running: Arc<RwLock<bool>>,
    queue: Arc<Mutex<Vec<Box<dyn Fn() + Send + 'static>>>>,
}

impl ThreadPool {
    pub fn new(workers: usize) -> ThreadPool {
        let running = Arc::new(RwLock::new(true));

        let queue = Arc::new(Mutex::new(Vec::<Box<dyn Fn() + 'static + Send>>::new()));

        let workers = (0..workers).into_iter().map(|s| {
            Worker {
                running: running.clone(),
                id: s,
                handle: spawn({
                    let queue = queue.clone();
                    let running = running.clone();
                    move || {
                        loop {
                            if let Ok(b) = running.try_read() {
                                if !*b {
                                    return;
                                }
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            let mut action = None;
                            if let Ok(mut queue) = queue.try_lock() {
                                if !queue.is_empty() {
                                    action = Some(queue.remove(0));
                                }
                            }
                            if let Some(action) = action {
                                action();
                            }
                        }
                    }
                }),
            }
        }).collect::<Vec<Worker>>();

        Self {
            workers,
            running,
            queue,
        }
    }
    pub fn queue_job<F: Fn() + Send + 'static>(&self, f: F) {
        self.queue.lock().unwrap().push(Box::new(f));
    }

    pub fn queue_async_job<F>(&self, f: F) where F: AsyncFn() + Send + 'static {
        self.queue_job({
            move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    f().await;
                });
            }
        });
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    pub fn test() {

    }
}
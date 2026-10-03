//! A small pool of threads for slow, blocking shell work (thumbnail and icon
//! extraction).
//!
//! Jobs run newest-first: when the user scrolls quickly, the thumbnails that
//! are on screen *now* were requested last and should be produced first.

use std::sync::{Arc, Condvar, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Default)]
struct Queue {
    jobs: Mutex<Vec<Job>>,
    ready: Condvar,
}

#[derive(Clone)]
pub struct Worker {
    queue: Arc<Queue>,
}

impl Worker {
    pub fn new(threads: usize) -> Self {
        let queue = Arc::new(Queue::default());
        for i in 0..threads.max(1) {
            let queue = queue.clone();
            thread::Builder::new()
                .name(format!("shell-worker-{i}"))
                .spawn(move || {
                    crate::platform::init_worker_thread();
                    loop {
                        let job = {
                            let mut jobs = queue.jobs.lock().unwrap();
                            loop {
                                if let Some(job) = jobs.pop() {
                                    break job;
                                }
                                jobs = queue.ready.wait(jobs).unwrap();
                            }
                        };
                        job();
                    }
                })
                .expect("failed to spawn worker thread");
        }
        Self { queue }
    }

    pub fn spawn(&self, job: impl FnOnce() + Send + 'static) {
        self.queue.jobs.lock().unwrap().push(Box::new(job));
        self.queue.ready.notify_one();
    }
}

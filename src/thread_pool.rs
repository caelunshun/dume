use std::thread;

/// Abstraction over a threadpool that can spawn tasks.
use flume::{Receiver, Sender};

pub trait ThreadPool: Send + Sync + 'static {
    fn spawn(&self, task: impl FnOnce() + Send + 'static);
}

trait FnBox: Send + 'static {
    fn call(self: Box<Self>);
}

impl<T> FnBox for T
where
    T: FnOnce() + Send + 'static,
{
    fn call(self: Box<Self>) {
        (*self)()
    }
}

/// A basic thread pool.
pub struct BasicThreadPool {
    tasks: Sender<Box<dyn FnBox>>,
}

impl BasicThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver) = flume::unbounded::<Box<dyn FnBox>>();

        for _ in 0..num_threads {
            let receiver = receiver.clone();
            thread::Builder::new()
                .name("dume-worker".into())
                .spawn(move || {
                    for task in receiver {
                        task.call();
                    }
                });
        }

        Self { tasks: sender }
    }
}

impl ThreadPool for BasicThreadPool {
    fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.tasks
            .send(Box::new(task))
            .expect("thread pool has shut down")
    }
}

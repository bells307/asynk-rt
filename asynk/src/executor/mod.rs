pub(crate) mod handle;

mod task;

use crate::tp::ThreadPool;
use crate::JoinHandle;
use parking_lot::Mutex;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, Wake};
use std::thread::{self, Thread};

use self::task::{BlockedOnTaskWaker, SpawnedTaskWaker, Task};

pub struct Executor {
    thread_pool: ThreadPool,
    block_on_thread: Mutex<Option<Thread>>,
}

static EXECUTOR: OnceLock<Executor> = OnceLock::new();

impl Executor {
    pub fn new(thread_pool: ThreadPool) -> Self {
        Self {
            thread_pool,
            block_on_thread: Mutex::new(None),
        }
    }

    pub fn get() -> &'static Executor {
        EXECUTOR.get().expect("executor is not set")
    }

    pub fn set_global(self) {
        EXECUTOR.set(self).ok();
    }

    pub fn block_on<T>(
        &self,
        fut: impl Future<Output = T> + Send + 'static,
    ) -> Result<T, BlockOnError>
    where
        T: Send + 'static,
    {
        *self.block_on_thread.lock() = Some(thread::current());

        let (task, mut jh) = Task::<T, BlockedOnTaskWaker>::new(fut);
        task.clone().wake();

        let main_waker = Arc::clone(&task).into();
        let mut cx = Context::from_waker(&main_waker);

        let mut jh = Pin::new(&mut jh);

        loop {
            // Check if main task is ready
            if let Poll::Ready(res) = jh.as_mut().poll(&mut cx) {
                return Ok(res?);
            }

            // Park this thread until main task become ready
            thread::park();
        }
    }

    pub fn spawn<T>(&self, fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
    where
        T: Send + 'static,
    {
        let (task, jh) = Task::<T, SpawnedTaskWaker>::new(fut);

        // Wake the task so that it starts trying to complete
        task.wake();
        jh
    }

    fn unpark_blocked_thread(&self) {
        self.block_on_thread
            .lock()
            .take()
            .expect("block on thread is not set")
            .unpark();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockOnError {
    #[error("join error: {0}")]
    Join(#[from] handle::JoinError),
}

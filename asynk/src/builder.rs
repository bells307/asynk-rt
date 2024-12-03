use crate::{reactor::Reactor, Executor};
use std::{io, num::NonZeroUsize};
use tpool::ThreadPool;
use zeet::WorkStealThreadPool;

#[derive(Default)]
pub struct AsynkBuilder {
    task_threads: Option<NonZeroUsize>,
    blocking_threads: Option<NonZeroUsize>,
}

impl AsynkBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn task_threads(mut self, val: NonZeroUsize) -> Self {
        self.task_threads = Some(val);
        self
    }

    pub fn blocking_threads(mut self, val: NonZeroUsize) -> Self {
        self.blocking_threads = Some(val);
        self
    }

    pub fn build(self) -> io::Result<()> {
        let task_threads = self.task_threads.unwrap_or_else(Self::default_thread_count);

        let task_tp = WorkStealThreadPool::builder()
            .max_threads(task_threads)
            .build();

        let blocking_threads = self
            .blocking_threads
            .unwrap_or_else(Self::default_thread_count);

        let blocking_tp = ThreadPool::new(blocking_threads);

        Executor::new(task_tp, blocking_tp).set_global();
        Reactor::new()?.set_global();
        Ok(())
    }

    fn default_thread_count() -> NonZeroUsize {
        num_cpus::get().try_into().expect("can't define num cpus")
    }
}

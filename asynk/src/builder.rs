use std::io;

use crate::{reactor::Reactor, Executor, ThreadPool};

#[derive(Default)]
pub struct AsynkBuilder {
    thread_count: Option<usize>,
}

impl AsynkBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn worker_threads(mut self, val: usize) -> Self {
        let val = if val == 0 {
            Self::default_thread_count()
        } else {
            val
        };

        self.thread_count = Some(val);
        self
    }

    pub fn build(self) -> io::Result<()> {
        let thread_count = self.thread_count.unwrap_or_else(Self::default_thread_count);
        let thread_pool = ThreadPool::new(thread_count);
        Executor::new(thread_pool).set_global();
        Reactor::new()?.set_global();
        Ok(())
    }

    fn default_thread_count() -> usize {
        num_cpus::get()
    }
}

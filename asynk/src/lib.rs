mod builder;
mod executor;
mod net;
mod reactor;
mod tp;

use executor::Executor;
use std::future::Future;

pub use {
    builder::AsynkBuilder,
    executor::{handle::JoinHandle, BlockOnError},
    net::tcp::{stream::TcpStream, TcpListener},
    tp::ThreadPool,
};

pub fn builder() -> AsynkBuilder {
    AsynkBuilder::new()
}

pub fn block_on<T>(fut: impl Future<Output = T> + Send + 'static) -> Result<T, BlockOnError>
where
    T: Send + 'static,
{
    Executor::get().block_on(fut)
}

pub fn spawn<T>(fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    Executor::get().spawn(fut)
}

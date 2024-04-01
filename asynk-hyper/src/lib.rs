pub use asynk::net::TcpListener;

use asynk::net::TcpStream as AsynkTcpStream;
use futures::{AsyncRead, AsyncWrite};
use hyper::rt::{Executor, Read, ReadBufCursor, Write};
use std::{
    future::Future,
    io::Result,
    mem::MaybeUninit,
    pin::Pin,
    task::{ready, Context, Poll},
};

#[derive(Clone)]
pub struct AsynkExecutor;

impl<Fut> Executor<Fut> for AsynkExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        asynk::spawn(fut);
    }
}

/// TcpStream adapter for `hyper`
pub struct TcpStream(AsynkTcpStream);

impl From<AsynkTcpStream> for TcpStream {
    fn from(stream: AsynkTcpStream) -> Self {
        Self(stream)
    }
}

impl Read for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: ReadBufCursor<'_>,
    ) -> Poll<Result<()>> {
        unsafe {
            let b = buf.as_mut();
            let b = &mut *(b as *mut [MaybeUninit<u8>] as *mut [u8]);
            let n = ready!(Pin::new(&mut self.0).poll_read(cx, b))?;
            buf.advance(n);
        };

        Poll::Ready(Ok(()))
    }
}

impl Write for TcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

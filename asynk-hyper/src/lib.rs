use asynk::TcpStream;
use futures::{AsyncRead, AsyncWrite};
use hyper::rt::{Read, ReadBufCursor, Write};
use std::{
    io::Result,
    mem::MaybeUninit,
    pin::Pin,
    task::{ready, Context, Poll},
};

pub struct HyperTcpStream(TcpStream);

impl From<TcpStream> for HyperTcpStream {
    fn from(stream: TcpStream) -> Self {
        Self(stream)
    }
}

impl Read for HyperTcpStream {
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

impl Write for HyperTcpStream {
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

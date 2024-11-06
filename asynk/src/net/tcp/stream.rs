use crate::reactor::io_handle::IoHandle;
use futures::{AsyncRead, AsyncWrite};
use mio::{net::TcpStream as MioTcpStream, Interest};
use std::{
    io::Result,
    net::{Shutdown, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
};

/// A non-blocking TCP stream between a local socket and a remote socket.
///
/// The socket will be closed when the value is dropped.
pub struct TcpStream(IoHandle<MioTcpStream>);

impl TcpStream {
    pub(crate) fn new(io_handle: IoHandle<MioTcpStream>) -> Self {
        Self(io_handle)
    }

    /// Create a new TCP stream and issue a non-blocking connect to the
    /// specified address.
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = MioTcpStream::connect(addr)?;
        Ok(Self(IoHandle::try_new(
            stream,
            Interest::READABLE.add(Interest::WRITABLE),
        )?))
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpStream {
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

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(self.0.shutdown(Shutdown::Both))
    }
}

pub(crate) mod stream;

use super::TcpStream;
use crate::reactor::{
    direction::Direction,
    io_handle::{poll_io, IoHandle},
};
use futures::Stream;
use mio::{net::TcpListener as MioTcpListener, Interest};
use std::{
    io::{self, Result},
    net::SocketAddr,
    pin::Pin,
    task::{ready, Context, Poll},
};

/// A structure representing a TCP listener.
pub struct TcpListener(MioTcpListener);

impl TcpListener {
    /// Convenience method to bind a new TCP listener to the specified address
    /// to receive new connections.
    ///
    /// This function will take the following steps:
    ///
    /// 1. Create a new TCP socket.
    /// 2. Set the `SO_REUSEADDR` option on the socket on Unix.
    /// 3. Bind the socket to the specified address.
    /// 4. Calls `listen` on the socket to prepare it to receive new connections.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self(MioTcpListener::bind(addr)?))
    }

    /// Accepts a new `TcpStream`.
    ///
    /// If an accepted stream is returned, the remote address of the peer is
    /// returned along with it.
    pub fn accept(self) -> io::Result<Accept> {
        let io_handle = IoHandle::try_new(self.0, Interest::READABLE)?;
        Ok(Accept(io_handle))
    }
}

pub struct Accept(IoHandle<MioTcpListener>);

impl Stream for Accept {
    type Item = Result<(TcpStream, SocketAddr)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (stream, addr) = ready!(poll_io(self.0.token(), cx, Direction::Read, || self
            .0
            .accept()))?;

        let io_handle = IoHandle::try_new(stream, Interest::READABLE.add(Interest::WRITABLE))?;
        let tcp_stream = TcpStream::new(io_handle);
        Poll::Ready(Some(Ok((tcp_stream, addr))))
    }
}

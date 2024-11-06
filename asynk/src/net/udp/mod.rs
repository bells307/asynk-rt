pub(crate) mod stream;

use crate::reactor::{
    direction::Direction,
    io_handle::{poll_io, IoHandle},
};
use futures::future::poll_fn;
use mio::{net::UdpSocket as MioUdpSocket, Interest};
use std::{io, net::SocketAddr};

/// A User Datagram Protocol socket.
///
/// This is an implementation of a bound UDP socket. This supports both IPv4 and
/// IPv6 addresses, and there is no corresponding notion of a server because UDP
/// is a datagram protocol.
pub struct UdpSocket(IoHandle<MioUdpSocket>);

impl UdpSocket {
    /// Creates a UDP socket from the given address.
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = MioUdpSocket::bind(addr)?;
        Ok(Self(IoHandle::try_new(socket, Interest::READABLE)?))
    }

    // Connects the UDP socket setting the default destination for `send()`
    // and limiting packets that are read via `recv` from the address specified
    // in `addr`.
    pub async fn connect(&'static self, addr: SocketAddr) -> io::Result<()> {
        crate::spawn_blocking(move || self.0.connect(addr))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
    }

    /// Receives data from the socket previously bound with connect(). On success, returns
    /// the number of bytes read.
    ///
    /// # Notes
    ///
    /// On Windows, if the data is larger than the buffer specified, the buffer
    /// is filled with the first part of the data, and recv returns the error
    /// WSAEMSGSIZE(10040). The excess data is lost.
    /// Make sure to always use a sufficiently large buffer to hold the
    /// maximum UDP packet size, which can be up to 65536 bytes in size.
    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        poll_fn(|cx| poll_io(self.0.token(), cx, Direction::Read, || self.0.recv(buf))).await
    }

    /// Receives data from the socket. On success, returns the number of bytes
    /// read and the address from whence the data came.
    ///
    /// # Notes
    ///
    /// On Windows, if the data is larger than the buffer specified, the buffer
    /// is filled with the first part of the data, and recv_from returns the error
    /// WSAEMSGSIZE(10040). The excess data is lost.
    /// Make sure to always use a sufficiently large buffer to hold the
    /// maximum UDP packet size, which can be up to 65536 bytes in size.
    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        poll_fn(|cx| {
            poll_io(self.0.token(), cx, Direction::Read, || {
                self.0.recv_from(buf)
            })
        })
        .await
    }

    /// Sends data on the socket to the address previously bound via connect(). On success,
    /// returns the number of bytes written.
    pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        poll_fn(|cx| poll_io(self.0.token(), cx, Direction::Write, || self.0.send(buf))).await
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub async fn send_to(&self, buf: &[u8], target: SocketAddr) -> io::Result<usize> {
        poll_fn(|cx| {
            poll_io(self.0.token(), cx, Direction::Write, || {
                self.0.send_to(buf, target)
            })
        })
        .await
    }

    /// Receives data from the socket, without removing it from the input queue.
    /// On success, returns the number of bytes read.
    ///
    /// # Notes
    ///
    /// On Windows, if the data is larger than the buffer specified, the buffer
    /// is filled with the first part of the data, and peek returns the error
    /// WSAEMSGSIZE(10040). The excess data is lost.
    /// Make sure to always use a sufficiently large buffer to hold the
    /// maximum UDP packet size, which can be up to 65536 bytes in size.
    pub async fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        poll_fn(|cx| poll_io(self.0.token(), cx, Direction::Read, || self.0.peek(buf))).await
    }

    /// Receives data from the socket, without removing it from the input queue.
    /// On success, returns the number of bytes read and the address from whence
    /// the data came.
    ///
    /// # Notes
    ///
    /// On Windows, if the data is larger than the buffer specified, the buffer
    /// is filled with the first part of the data, and peek_from returns the error
    /// WSAEMSGSIZE(10040). The excess data is lost.
    /// Make sure to always use a sufficiently large buffer to hold the
    /// maximum UDP packet size, which can be up to 65536 bytes in size.
    pub async fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        poll_fn(|cx| {
            poll_io(self.0.token(), cx, Direction::Read, || {
                self.0.peek_from(buf)
            })
        })
        .await
    }
}

pub(crate) mod stream;

use crate::reactor::{direction::Direction, io_handle::IoHandle};
use mio::net::UdpSocket as MioUdpSocket;
use std::{future::poll_fn, io, net::SocketAddr, task::Poll};

pub struct UdpSocket(IoHandle<MioUdpSocket>);

impl UdpSocket {
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = MioUdpSocket::bind(addr)?;
        Ok(Self(IoHandle::new(socket)))
    }

    pub async fn connect(&'static self, addr: SocketAddr) -> io::Result<()> {
        crate::spawn_blocking(move || self.0.source().connect(addr))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
    }

    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<()> {
        //     let s = poll_fn(|cx| {
        //         if !self.0.check_direction(Direction::Read) {
        //             self.0
        //                 .register_direction(Direction::Read, cx.waker().clone())?;
        //             return Poll::Pending;
        //         }
        //
        //         Poll::Ready(Ok::<_, io::Error>(()))
        //     });
        todo!()
        //     // self.0.recv(buf);
    }
}

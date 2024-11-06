pub(crate) mod stream;

use mio::net::UdpSocket as MioUdpSocket;
use std::{io, net::SocketAddr};

pub struct UdpSocket(MioUdpSocket);

impl UdpSocket {
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self(MioUdpSocket::bind(addr)?))
    }

    pub async fn connect(self, addr: SocketAddr) -> io::Result<()> {
        crate::spawn_blocking(move || self.0.connect(addr))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
    }
}

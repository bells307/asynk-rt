use std::io::{self, Error};

use asynk::net::UdpSocket;

const SERVER_SOCK_ADDR: &str = "127.0.0.1:8040";

fn main() {
    asynk::builder().build().unwrap();

    asynk::block_on(async {
        let server = asynk::spawn(server());
        server.await.unwrap().unwrap();
    })
    .unwrap();
}

async fn server() -> io::Result<()> {
    let addr = SERVER_SOCK_ADDR.parse().map_err(Error::other)?;
    let sock = UdpSocket::bind(addr)?;

    let mut buf = [0; 1024];

    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;

        println!("{:?} bytes received from {:?}", len, addr);

        let len = sock.send_to(&buf[..len], addr).await?;
        println!("{:?} bytes sent", len);
    }
}

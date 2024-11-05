use asynk::net::{TcpListener, TcpStream};
use futures::{future, AsyncReadExt, AsyncWriteExt, StreamExt};
use futures_timer::Delay;
use std::{
    io::{self, Error},
    time::Duration,
};

const SERVER_SOCK_ADDR: &str = "127.0.0.1:8040";

const SERVER_RESPONSE: &str = "HTTP/1.1 200 OK
Content-Type: text/html
Connection: keep-alive
Content-Length: 23

<h1>Hello, world!</h1>
";

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

    let listener = TcpListener::bind(addr)?;
    let mut accept = listener.accept();

    let Some(res) = accept.next().await else {
        return Ok(());
    };

    let (mut stream, _) = res?;

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await?;
    println!("{n}");

    Ok(())
}

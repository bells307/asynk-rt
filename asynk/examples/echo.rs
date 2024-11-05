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

    let is_double_crnl = |window: &[u8]| {
        window.len() >= 4
            && (window[0] == b'\r')
            && (window[1] == b'\n')
            && (window[2] == b'\r')
            && (window[3] == b'\n')
    };

    while let Some(res) = accept.next().await {
        // Spawn new task for the connection
        asynk::spawn(async move {
            // Accept the connection
            let (mut stream, _) = res?;

            loop {
                let mut buf = [0; 1024];

                if stream.read(&mut buf).await? == 0 {
                    break;
                }

                stream.write_all(&buf).await?;
            }

            Ok::<_, Error>(())
        });
    }

    Ok(())
}

async fn client() -> io::Result<()> {
    let addr = SERVER_SOCK_ADDR.parse().map_err(Error::other)?;

    loop {
        let mut stream = TcpStream::connect(addr)?;

        let req = format!("GET / HTTP/1.1\r\nHost:{}\r\n\r\n", addr);
        stream.write_all(req.as_bytes()).await?;

        let mut resp = String::with_capacity(128);
        stream.read_to_string(&mut resp).await?;

        println!("resp: {resp}");

        stream.flush().await?;

        Delay::new(Duration::from_millis(100)).await;
    }
}

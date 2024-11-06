use asynk::net::TcpListener;
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use std::io::{self, Error};

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
        server.await.unwrap().unwrap()
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

            let mut buf = [0; 1024];
            // HTTP request data
            let mut data = Vec::with_capacity(128);

            loop {
                let read = stream.read(&mut buf).await?;

                data.extend(&buf[0..read]);

                if data.windows(4).any(is_double_crnl) || read == 0 {
                    break;
                }
            }

            stream.write_all(SERVER_RESPONSE.as_bytes()).await?;

            stream.flush().await?;

            Ok::<_, Error>(())
        });
    }

    Ok(())
}

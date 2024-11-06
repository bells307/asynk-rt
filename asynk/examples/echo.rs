use asynk::net::TcpListener;
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use std::io::{self, Error};

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

    let listener = TcpListener::bind(addr)?;
    let mut accept = listener.accept()?;

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

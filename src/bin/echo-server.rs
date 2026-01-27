use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let host = "127.0.0.1";
    let port = 6142;
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    // Connection closed
                    Ok(0) => return,
                    Ok(n) => {
                        println!("Received message {:?}", &buf[..n]);
                        if socket.write_all(&buf[..n]).await.is_err() {
                            return;
                        }
                    }
                    // Unexpected error
                    Err(_) => return,
                }
            }
        });
    }
}

use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

/// Multiple different commands are multiplexed over a single channel
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

/// Provided by the requester and used by the manager task
/// to send the response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    // Create a new channel with capacity of 32
    let (tx, mut rx) = mpsc::channel(32);

    let manager = tokio::spawn(async move {
        let host = "127.0.0.1";
        let port = 6379;
        let mut client = client::connect(format!("{}:{}", host, port)).await.unwrap();
        println!("Connected to server {} on port {}", host, port);

        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    // The `sender` handle will be moved to the first task
    // We need to create a second `sender` first
    let tx2 = tx.clone();

    // Spawn two tasks, one gets a key, the other sets a key
    let task1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        tx.send(cmd).await.unwrap();

        let resp = resp_rx.await;
        println!("Got response: {:?}", resp);
    });

    let task2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };

        tx2.send(cmd).await.unwrap();

        let resp = resp_rx.await;
        println!("Got response: {:?}", resp);
    });

    task1.await.unwrap();
    task2.await.unwrap();
    manager.await.unwrap();
}

use mini_redis::client;
use tokio_stream::StreamExt;

async fn publish() -> mini_redis::Result<()> {
    let host = "127.0.0.1";
    let port = 6379;
    let mut client = client::connect(format!("{}:{}", host, port)).await?;

    client.publish("numbers", "1".into()).await?;
    client.publish("numbers", "two".into()).await?;
    client.publish("numbers", "3".into()).await?;
    client.publish("numbers", "four".into()).await?;
    client.publish("numbers", "5".into()).await?;

    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let host = "127.0.0.1";
    let port = 6379;
    let client = client::connect(format!("{}:{}", host, port)).await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;

    let messages = subscriber
        .into_stream()
        .filter(|msg| match msg {
            Ok(msg) if msg.content.len() == 1 => true,
            _ => false,
        })
        .take(3)
        .map(|msg| msg.unwrap().content);

    tokio::pin!(messages);

    while let Some(msg) = messages.next().await {
        println!("Got {:?}", msg);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    tokio::spawn(async { publish().await });

    subscribe().await?;

    println!("DONE");
    Ok(())
}

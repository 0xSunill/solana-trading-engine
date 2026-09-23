use tokio::sync::watch;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = watch::channel(100);

    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        sender.send(200).unwrap();

        sleep(Duration::from_secs(1)).await;
        sender.send(300).unwrap();

        sleep(Duration::from_secs(1)).await;
        sender.send(400).unwrap();
    });

    while receiver.changed().await.is_ok() {
        println!("New value: {}", *receiver.borrow());
    }
}
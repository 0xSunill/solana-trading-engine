use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (market_tx, mut market_rx) = mpsc::channel(10);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    tokio::spawn(async move {
        for i in 0..10 {
            market_tx.send(i).await.unwrap();
            sleep(Duration::from_millis(500)).await;
        }
    });

    tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        shutdown_tx.send(()).await.unwrap();
    });

    loop {
        tokio::select! {
            Some(price) = market_rx.recv() => {
                println!("Market price: {}", price);
            }

            Some(_) = shutdown_rx.recv() => {
                println!("Shutdown received");
                break;
            }
        }
    }

    println!("Engine stopped");
}
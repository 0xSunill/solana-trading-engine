// use tokio::sync::broadcast;

// #[derive(Debug, Clone)]
// struct MarketEvent {
//     symbol: String,
//     price: f64,
// }

// #[tokio::main]
// async fn main() {
//     let (sender, mut strategy_rx) = broadcast::channel(10);

//     let mut logger_rx = sender.subscribe();
//     let mut metrics_rx = sender.subscribe();

//     sender.send(MarketEvent {
//         symbol: "SOL/USDC".to_string(),
//         price: 205.50,
//     }).unwrap();

//     println!("Strategy: {:?}", strategy_rx.recv().await.unwrap());
//     println!("Logger: {:?}", logger_rx.recv().await.unwrap());
//     println!("Metrics: {:?}", metrics_rx.recv().await.unwrap());
// }




// use tokio::sync::broadcast;

// #[tokio::main]
// async fn main() {
//     let (sender, mut strategy_rx) = broadcast::channel(10);

//     let mut logger_rx = sender.subscribe();
//     let mut monitor_rx = sender.subscribe();

//     let strategy = tokio::spawn(async move {
//         while let Ok(event) = strategy_rx.recv().await {
//             println!("Strategy received: {}", event);
//         }
//     });

//     let logger = tokio::spawn(async move {
//         while let Ok(event) = logger_rx.recv().await {
//             println!("Logger received: {}", event);
//         }
//     });

//     let monitor = tokio::spawn(async move {
//         while let Ok(event) = monitor_rx.recv().await {
//             println!("Monitor received: {}", event);
//         }
//     });

//     for i in 0..5 {
//         sender.send(format!("Market event {}", i)).unwrap();
//     }

//     drop(sender);

//     strategy.await.unwrap();
//     logger.await.unwrap();
//     monitor.await.unwrap();
// }






use tokio::sync::{mpsc, oneshot};

struct Request {
    value: u64,
    response: oneshot::Sender<u64>,
}

#[tokio::main]
async fn main() {
    let (request_tx, mut request_rx) = mpsc::channel::<Request>(10);

    let worker = tokio::spawn(async move {
        while let Some(request) = request_rx.recv().await {
            let result = request.value * 2;

            request.response.send(result).unwrap();
        }
    });

    let (response_tx, response_rx) = oneshot::channel();

    request_tx
        .send(Request {
            value: 50,
            response: response_tx,
        })
        .await
        .unwrap();

    let result = response_rx.await.unwrap();

    println!("Result = {}", result);

    drop(request_tx);

    worker.await.unwrap();
}
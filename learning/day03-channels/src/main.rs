// use tokio::sync::mpsc;

// #[tokio::main]
// async fn main() {
//     let (sender, mut receiver) = mpsc::channel(10);

//     let producer = tokio::spawn(async move {
//         sender.send(100).await.unwrap();
//         sender.send(200).await.unwrap();
//         sender.send(300).await.unwrap();
        
//     });

//     let consumer = tokio::spawn(async move {
//         while let Some(value) = receiver.recv().await {
//             println!("Received: {}", value);
//         }
//     });

//     producer.await.unwrap();
//     consumer.await.unwrap();
// }


use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(10);

    let sender_a = sender.clone();
    let sender_b = sender.clone();

    let task_a = tokio::spawn(async move {
        sender_a.send("Price from feed A").await.unwrap();
    });

    let task_b = tokio::spawn(async move {
        sender_b.send("Price from feed B").await.unwrap();
    });

    let first = receiver.recv().await.unwrap();
    let second = receiver.recv().await.unwrap();

    println!("{}", first);
    println!("{}", second);

    task_a.await.unwrap();
    task_b.await.unwrap();
}
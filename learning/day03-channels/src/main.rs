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
//     producer.await.unwrap();
//     consumer.await.unwrap();
// }
//     let sender_a = sender.clone();
//     let sender_b = sender.clone();
//     let task_a = tokio::spawn(async move {
//         sender_a.send("Price from feed A").await.unwrap();
//     let task_b = tokio::spawn(async move {
//         sender_b.send("Price from feed B").await.unwrap();
//     let first = receiver.recv().await.unwrap();
//     let second = receiver.recv().await.unwrap();
//     println!("{}", first);
//     println!("{}", second);
//     task_a.await.unwrap();
//     task_b.await.unwrap();

// use tokio::sync::mpsc;

// #[tokio::main]
// async fn main() {
//     let (sender, mut receiver) = mpsc::channel(10);

//     let sender_a = sender.clone();
//     let sender_b = sender.clone();
//     let sender_c = sender.clone();

//     // We don't need the original sender anymore.
//     drop(sender);

//     let task_a = tokio::spawn(async move {
//         for i in 1..=3 {
//             sender_a
//                 .send(format!("A {}", i))
//                 .await
//                 .unwrap();
//         }
//     });

//     let task_b = tokio::spawn(async move {
//         for i in 1..=3 {
//             sender_b
//                 .send(format!("B {}", i))
//                 .await
//                 .unwrap();
//         }
//     });

//     let task_c = tokio::spawn(async move {
//         for i in 1..=3 {
//             sender_c
//                 .send(format!("C {}", i))
//                 .await
//                 .unwrap();
//         }
//     });

//     let receiver_task = tokio::spawn(async move {
//         while let Some(value) = receiver.recv().await {
//             println!("{}", value);
//         }
//     });

//     task_a.await.unwrap();
//     task_b.await.unwrap();
//     task_c.await.unwrap();

//     receiver_task.await.unwrap();
// }








// use tokio::sync::mpsc;
// use tokio::time::{sleep, Duration};

// #[tokio::main]
// async fn main() {
//     let (sender, mut receiver) = mpsc::channel(2);

//     let producer = tokio::spawn(async move {
//         for i in 1..=10 {
//             println!("Trying to send {}", i);

//             sender.send(i).await.unwrap();

//             println!("Sent {}", i);
//         }
//     });

//     let consumer = tokio::spawn(async move {
//         sleep(Duration::from_secs(3)).await;

//         while let Some(value) = receiver.recv().await {
//             println!("Received {}", value);
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

    drop(sender);

    sender_a.send(100).await.unwrap();

    drop(sender_a);

    while let Some(value) = receiver.recv().await {
        println!("Received: {}", value);
    }

    println!("Done");
}
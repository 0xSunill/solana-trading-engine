// use tokio::time::{sleep,Duration};

// async fn get_price() -> f64 {
//     println!("Starting price request...");
//     sleep(Duration::from_secs(3)).await;
//     println!("Done!");
//     19.99
// }

// #[tokio::main]
// async fn main() {
//     let price = get_price().await;

//     println!("Price: {}", price);
// }

// use tokio::time::{sleep, Duration};

// async fn task_a() {
//     println!("Task A started");

//     sleep(Duration::from_secs(3)).await;

//     println!("Task A finished");
// }

// async fn task_b() {
//     println!("Task B started");

//     sleep(Duration::from_secs(1)).await;

//     println!("Task B finished");
// }

// #[tokio::main]
// async fn main() {
//     // let handle_a = tokio::spawn(task_a());
//     // let handle_b = tokio::spawn(task_b());

//     // handle_a.await.unwrap();
//     // handle_b.await.unwrap();
//     task_a().await;
//     task_b().await;

//     println!("All tasks finished");
// }

#[tokio::main]
async fn main() {
    let message = String::from("Hello from Tokio");

    let handle = tokio::spawn(async move {
        println!("{}", message);
    });

    handle.await.unwrap();
}
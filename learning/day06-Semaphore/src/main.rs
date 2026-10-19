use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration, Instant};

async fn fetch_account(id: u64, started: Instant) {
    println!(
        "Started account {id} at {:.2}s",
        started.elapsed().as_secs_f64()
    );

    // Simulate an RPC request taking 1 second.
    sleep(Duration::from_secs(1)).await;

    println!("Finished account {id}");
}

#[tokio::main]
async fn main() {
    const MAX_IN_FLIGHT: usize = 3;

    let semaphore = Arc::new(Semaphore::new(MAX_IN_FLIGHT));
    let started = Instant::now();

    let mut tasks = JoinSet::new();

    for id in 1..=10 {
        // Concurrency limit: acquire a permit before starting work.
        let permit = Arc::clone(&semaphore)
            .acquire_owned()
            .await
            .unwrap();

        tasks.spawn(async move {
            // Keep the permit alive throughout the operation.
            let _permit = permit;
            fetch_account(id, started).await;
        });

        // Rate limit: pace operation starts.
        if id < 10 {
            sleep(Duration::from_millis(200)).await;
        }
    }

    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            eprintln!("Task failed: {error}");
        }
    }

    println!("All accounts processed");
}
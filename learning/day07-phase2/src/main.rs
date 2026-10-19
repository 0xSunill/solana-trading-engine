use rand::Rng;
use std::future::Future;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum FetchError {
    Temporary,
    Permanent,
}

impl FetchError {
    fn is_retryable(&self) -> bool {
        matches!(self, FetchError::Temporary)
    }
}

async fn fetch_data(
    attempts: Arc<AtomicUsize>,
) -> Result<u64, FetchError> {
    let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;

    println!("Request attempt {attempt}");

    // Simulate temporary failures on the first 3 attempts.
    if attempt <= 3 {
        Err(FetchError::Temporary)
    } else {
        Ok(42)
    }
}

async fn retry_with_backoff<F, Fut>(
    mut operation: F,
    max_attempts: u32,
) -> Result<u64, FetchError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<u64, FetchError>>,
{
    assert!(max_attempts > 0);

    let base_delay = Duration::from_millis(200);
    let max_delay = Duration::from_secs(3);

    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(value) => return Ok(value),

            Err(error) if !error.is_retryable() => {
                return Err(error);
            }

            Err(error) if attempt == max_attempts => {
                println!("Maximum attempts reached");
                return Err(error);
            }

            Err(error) => {
                let multiplier = 2_u32.saturating_pow(attempt - 1);

                let backoff = base_delay
                    .saturating_mul(multiplier)
                    .min(max_delay);

                // Add random jitter of 0–150 milliseconds.
                let jitter_ms =
                    rand::thread_rng().gen_range(0..=150);

                let delay = backoff + Duration::from_millis(jitter_ms);

                println!(
                    "Temporary failure: {error:?}. \
                     Retrying in {:.2} seconds",
                    delay.as_secs_f64()
                );

                sleep(delay).await;
            }
        }
    }

    unreachable!("max_attempts must be greater than zero")
}

#[tokio::main]
async fn main() {
    let attempts = Arc::new(AtomicUsize::new(0));

    let result = retry_with_backoff(
        || fetch_data(Arc::clone(&attempts)),
        5,
    )
    .await;

    match result {
        Ok(value) => println!("Success! Data: {value}"),
        Err(error) => println!("Request failed: {error:?}"),
    }
}
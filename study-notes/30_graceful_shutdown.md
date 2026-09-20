# Chapter 30: Graceful Shutdown

> **Connection to the bigger picture**: In production, your Solana backend needs to shut down cleanly — finish in-flight requests, close database connections, stop WebSocket listeners, and save state. An abrupt kill can lose transactions, corrupt data, or leave orders in limbo.

---

## The Shutdown Problem

```
Running Server:
├── HTTP requests being handled
├── WebSocket connections active
├── Background tasks running
├── Database transactions in progress
├── Solana transactions being submitted
└── Market data being processed

SIGTERM received! What now?
├── Stop accepting new requests
├── Finish in-flight requests
├── Close WebSocket connections gracefully
├── Complete pending database transactions
├── Wait for Solana transaction confirmations
├── Save any cached state
└── Then exit
```

---

## Using `CancellationToken`

`tokio_util::sync::CancellationToken` is the standard pattern:

```rust
use tokio_util::sync::CancellationToken;
use tokio::signal;

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    
    // Background task respects cancellation
    let task_token = token.clone();
    let bg_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = task_token.cancelled() => {
                    println!("Background task shutting down...");
                    // Cleanup work here
                    break;
                }
                _ = do_periodic_work() => {
                    println!("Work done");
                }
            }
        }
    });
    
    // Wait for shutdown signal
    signal::ctrl_c().await.unwrap();
    println!("Shutdown signal received");
    
    // Signal all tasks to stop
    token.cancel();
    
    // Wait for tasks to finish
    bg_task.await.unwrap();
    println!("Clean shutdown complete");
}

async fn do_periodic_work() {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
}
```

---

## Graceful Shutdown with Axum

```rust
use axum::Router;
use tokio::signal;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", axum::routing::get(|| async { "OK" }));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    
    println!("Server shut down gracefully");
}

async fn shutdown_signal() {
    signal::ctrl_c().await.unwrap();
    println!("Received Ctrl+C, starting shutdown...");
}
```

---

## Complete Shutdown Architecture

```rust
use tokio_util::sync::CancellationToken;
use tokio::sync::mpsc;

struct App {
    shutdown_token: CancellationToken,
}

impl App {
    async fn run(&self) {
        let token = self.shutdown_token.clone();
        
        // Start all components
        let ws_handle = tokio::spawn(self.run_websocket_listener(token.clone()));
        let processor_handle = tokio::spawn(self.run_event_processor(token.clone()));
        let server_handle = tokio::spawn(self.run_http_server(token.clone()));
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await.unwrap();
        println!("Initiating graceful shutdown...");
        
        // Signal all components to stop
        token.cancel();
        
        // Wait for all components to finish (with timeout)
        let timeout = tokio::time::Duration::from_secs(30);
        tokio::select! {
            _ = async {
                let _ = tokio::join!(ws_handle, processor_handle, server_handle);
            } => {
                println!("All components shut down cleanly");
            }
            _ = tokio::time::sleep(timeout) => {
                println!("Shutdown timed out — forcing exit");
            }
        }
    }
    
    async fn run_websocket_listener(&self, token: CancellationToken) {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    println!("WebSocket listener stopping...");
                    break;
                }
                // ... listen for events
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
            }
        }
    }
    
    async fn run_event_processor(&self, token: CancellationToken) {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    println!("Event processor stopping...");
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
            }
        }
    }
    
    async fn run_http_server(&self, token: CancellationToken) {
        // ... serve HTTP until cancelled
        token.cancelled().await;
        println!("HTTP server stopping...");
    }
}
```

---

## Things to Remember

1. **`CancellationToken`** is the standard shutdown signaling mechanism.
2. **`select!` with cancellation** — check for shutdown in every long-running loop.
3. **Axum has built-in graceful shutdown** — `with_graceful_shutdown()`.
4. **Set a shutdown timeout** — don't wait forever for tasks to finish.
5. **Close channels** — dropping senders signals receivers to stop.
6. **Drain work queues** — process remaining items before exiting.

---

> **Next**: [Chapter 31: Tracing and Debugging](./31_tracing_debugging.md)

# Chapter 33: Real-Time Market Data Processing

> **Connection to the bigger picture**: A Solana trading system must subscribe to market data, process events in real-time, update internal state, execute strategy logic, and submit transactions — all concurrently with low latency.

---

## The Event Processing Pipeline

```
Solana WebSocket
      │
      ▼
┌──────────────┐    mpsc     ┌──────────────┐    mpsc     ┌──────────────┐
│ WS Listener  │───────────▶│ Parser /     │───────────▶│ Strategy     │
│ (async task) │  raw events │ Normalizer   │  parsed    │ Engine       │
└──────────────┘             └──────────────┘  events    └──────┬───────┘
                                                                │
                                                          mpsc  │ orders
                                                                ▼
                                                         ┌──────────────┐
                                                         │ Transaction  │
                                                         │ Submitter    │
                                                         └──────────────┘
```

---

## Implementing the Pipeline

```rust
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;

// Event types
#[derive(Debug, Clone)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    timestamp: u64,
}

#[derive(Debug)]
struct TradeSignal {
    symbol: String,
    action: String,  // "buy" or "sell"
    price: f64,
    quantity: f64,
}

// Shared state
struct MarketState {
    prices: std::collections::HashMap<String, f64>,
    positions: std::collections::HashMap<String, f64>,
}

async fn run_trading_system() -> anyhow::Result<()> {
    let (event_tx, mut event_rx) = mpsc::channel::<PriceUpdate>(1000);
    let (signal_tx, mut signal_rx) = mpsc::channel::<TradeSignal>(100);
    
    let state = Arc::new(RwLock::new(MarketState {
        prices: std::collections::HashMap::new(),
        positions: std::collections::HashMap::new(),
    }));
    
    let shutdown = tokio_util::sync::CancellationToken::new();
    
    // Task 1: WebSocket listener (producer)
    let event_tx_clone = event_tx.clone();
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_clone.cancelled() => break,
                // Simulated — in real code, read from WebSocket
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    let update = PriceUpdate {
                        symbol: "SOL/USD".into(),
                        price: 150.0 + rand_offset(),
                        timestamp: now(),
                    };
                    if event_tx_clone.send(update).await.is_err() {
                        break;
                    }
                }
            }
        }
    });
    
    // Task 2: Strategy engine (consumer + producer)
    let state_clone = Arc::clone(&state);
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_clone.cancelled() => break,
                Some(event) = event_rx.recv() => {
                    // Update state
                    {
                        let mut state = state_clone.write().await;
                        state.prices.insert(event.symbol.clone(), event.price);
                    }
                    
                    // Run strategy (read-only state access)
                    let state = state_clone.read().await;
                    if let Some(signal) = evaluate_strategy(&state, &event) {
                        signal_tx.send(signal).await.ok();
                    }
                }
            }
        }
    });
    
    // Task 3: Transaction submitter
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_clone.cancelled() => break,
                Some(signal) = signal_rx.recv() => {
                    tracing::info!(
                        symbol = %signal.symbol,
                        action = %signal.action,
                        price = signal.price,
                        "Executing trade signal"
                    );
                    // Build and submit Solana transaction...
                }
            }
        }
    });
    
    // Wait for shutdown
    tokio::signal::ctrl_c().await?;
    shutdown.cancel();
    
    Ok(())
}

fn evaluate_strategy(state: &MarketState, event: &PriceUpdate) -> Option<TradeSignal> {
    // Simple example: buy if price drops below threshold
    if event.price < 145.0 {
        Some(TradeSignal {
            symbol: event.symbol.clone(),
            action: "buy".into(),
            price: event.price,
            quantity: 1.0,
        })
    } else {
        None
    }
}

fn rand_offset() -> f64 { 0.0 }  // Placeholder
fn now() -> u64 { 0 }  // Placeholder
```

---

## Concurrency Considerations

| Concern | Solution |
|---------|----------|
| Events arriving faster than processing | Bounded channels with backpressure |
| Multiple strategies reading prices | `Arc<RwLock<State>>` — many readers |
| Price updates must be sequential | Single consumer task per symbol |
| Transaction submission is slow | Separate submitter task, don't block strategy |
| System must shut down cleanly | CancellationToken, select! |
| Logging across tasks | tracing with spans |

---

> **Next**: [Chapter 34: Low-Latency System Concepts](./34_low_latency_systems.md)

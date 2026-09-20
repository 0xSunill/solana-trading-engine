# Chapter 24: Concurrency Design Patterns

> **Connection to the bigger picture**: These patterns are the building blocks of concurrent systems. Your Solana trading backend will use all of them: producer/consumer for market data pipelines, worker pools for parallel processing, shared state for order books, and actor models for component isolation.

---

## Producer / Consumer

```
Producer ──▶ Channel ──▶ Consumer
```

The most fundamental pattern: one component produces data, another consumes it.

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    // Producer
    tokio::spawn(async move {
        loop {
            let event = fetch_market_event().await;
            if tx.send(event).await.is_err() {
                break;  // Receiver dropped
            }
        }
    });
    
    // Consumer
    while let Some(event) = rx.recv().await {
        process_event(&event).await;
    }
}

async fn fetch_market_event() -> String { "price_update".into() }
async fn process_event(event: &str) { println!("Processing: {}", event); }
```

### Multi-stage pipeline

```
WebSocket ──▶ Parser ──▶ Strategy ──▶ Executor
           ch1        ch2          ch3
```

```rust
use tokio::sync::mpsc;

struct RawEvent(String);
struct ParsedEvent { symbol: String, price: f64 }
struct Order { symbol: String, amount: f64 }

#[tokio::main]
async fn main() {
    let (raw_tx, mut raw_rx) = mpsc::channel::<RawEvent>(100);
    let (parsed_tx, mut parsed_rx) = mpsc::channel::<ParsedEvent>(100);
    let (order_tx, mut order_rx) = mpsc::channel::<Order>(100);
    
    // Stage 1: Parser
    tokio::spawn(async move {
        while let Some(raw) = raw_rx.recv().await {
            let parsed = ParsedEvent { 
                symbol: "SOL".into(), 
                price: 150.0 
            };
            parsed_tx.send(parsed).await.ok();
        }
    });
    
    // Stage 2: Strategy
    tokio::spawn(async move {
        while let Some(event) = parsed_rx.recv().await {
            if event.price < 100.0 {
                let order = Order { 
                    symbol: event.symbol, 
                    amount: 1.0 
                };
                order_tx.send(order).await.ok();
            }
        }
    });
    
    // Stage 3: Executor
    tokio::spawn(async move {
        while let Some(order) = order_rx.recv().await {
            println!("Executing order: {} {}", order.symbol, order.amount);
        }
    });
    
    // Feed data into the pipeline
    for i in 0..10 {
        raw_tx.send(RawEvent(format!("event_{}", i))).await.ok();
    }
}
```

---

## Worker Pool

```
        ┌─ Worker 1
Queue ──┼─ Worker 2
        ├─ Worker 3
        └─ Worker 4
```

Multiple workers consuming from a shared queue for parallel processing.

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<u64>(100);
    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
    
    // Spawn worker pool
    let mut handles = vec![];
    for id in 0..4 {
        let rx = rx.clone();
        handles.push(tokio::spawn(async move {
            loop {
                let msg = {
                    let mut rx = rx.lock().await;
                    rx.recv().await
                };
                match msg {
                    Some(task) => {
                        println!("Worker {}: processing task {}", id, task);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                    None => break,  // Channel closed
                }
            }
        }));
    }
    
    // Send tasks
    for i in 0..20 {
        tx.send(i).await.unwrap();
    }
    drop(tx);  // Close channel
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

---

## Shared State Pattern

```
Task 1 ─┐
Task 2 ─┼─ Arc<Mutex<State>>
Task 3 ─┘
```

Multiple tasks accessing shared mutable state through locks.

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

type PriceMap = Arc<RwLock<HashMap<String, f64>>>;

async fn update_price(prices: PriceMap, symbol: String, price: f64) {
    prices.write().await.insert(symbol, price);
}

async fn get_price(prices: PriceMap, symbol: &str) -> Option<f64> {
    prices.read().await.get(symbol).copied()
}
```

---

## Actor Model

Each component ("actor") owns its state and communicates exclusively through channels.

```rust
use tokio::sync::{mpsc, oneshot};

// Messages the actor can receive
enum OrderBookMessage {
    UpdatePrice { symbol: String, price: f64 },
    GetPrice { symbol: String, respond_to: oneshot::Sender<Option<f64>> },
}

// The actor
struct OrderBookActor {
    receiver: mpsc::Receiver<OrderBookMessage>,
    prices: std::collections::HashMap<String, f64>,
}

impl OrderBookActor {
    fn new(receiver: mpsc::Receiver<OrderBookMessage>) -> Self {
        Self { receiver, prices: std::collections::HashMap::new() }
    }
    
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                OrderBookMessage::UpdatePrice { symbol, price } => {
                    self.prices.insert(symbol, price);
                }
                OrderBookMessage::GetPrice { symbol, respond_to } => {
                    let _ = respond_to.send(self.prices.get(&symbol).copied());
                }
            }
        }
    }
}

// Handle for communicating with the actor
#[derive(Clone)]
struct OrderBookHandle {
    sender: mpsc::Sender<OrderBookMessage>,
}

impl OrderBookHandle {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let actor = OrderBookActor::new(receiver);
        tokio::spawn(actor.run());
        Self { sender }
    }
    
    async fn update_price(&self, symbol: String, price: f64) {
        self.sender.send(OrderBookMessage::UpdatePrice { symbol, price })
            .await.unwrap();
    }
    
    async fn get_price(&self, symbol: &str) -> Option<f64> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(OrderBookMessage::GetPrice {
            symbol: symbol.to_string(),
            respond_to: tx,
        }).await.unwrap();
        rx.await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    let order_book = OrderBookHandle::new();
    
    order_book.update_price("SOL/USD".into(), 150.0).await;
    
    let price = order_book.get_price("SOL/USD").await;
    println!("SOL price: {:?}", price);  // Some(150.0)
}
```

> **Why the actor model is powerful for trading systems**: Each actor owns its state — no locks needed. All communication is through channels — no data races. Actors can be independently tested, restarted, and scaled. The order book actor processes updates sequentially, ensuring consistency without locks.

---

## Pattern Comparison

| Pattern | State ownership | Communication | Best for |
|---------|----------------|---------------|----------|
| Producer/Consumer | Transferred via channel | Unidirectional | Data pipelines |
| Worker Pool | No shared state | Channel (queue) | Parallel processing |
| Shared State | Shared via Arc<Mutex> | Direct access | Simple shared data |
| Actor | Owned by actor | Bidirectional channels | Complex components |

---

> **Next**: [Chapter 25: Async Rust for Backend Development](./25_async_backend_development.md)

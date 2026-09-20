# Chapter 37: Practical Projects

> **Connection to the bigger picture**: Theory without practice is useless. These 10 projects progressively build your skills from basic concurrency to a Solana trading engine prototype. Each project uses concepts from previous chapters.

---

## Project 1: Multithreaded Counter

**Goal**: Increment a counter from 10 threads simultaneously.

**Concepts practiced**: `thread::spawn`, `move`, `Arc`, `Mutex`, `JoinHandle`

**Requirements**:
- Spawn 10 threads, each incrementing a shared counter 1000 times
- Final count should be exactly 10,000
- Print the final count after all threads complete

**Architecture**:
```
Thread 0 ─┐
Thread 1 ─┤
Thread 2 ─┤
...       ─┼── Arc<Mutex<u64>> (counter)
Thread 9 ─┘
```

**Implementation steps**:
1. Create `Arc<Mutex<u64>>` initialized to 0
2. Spawn 10 threads, each cloning the Arc
3. Each thread locks the mutex, increments, releases 1000 times
4. Join all threads
5. Print the final value

**Common mistakes**:
- Forgetting `Arc::clone` (trying to move the same Arc into multiple threads)
- Forgetting `move` on the closure
- Holding the lock longer than needed

**Extensions**: Use `AtomicU64` instead of `Mutex` for better performance.

---

## Project 2: Thread-Based Producer/Consumer

**Goal**: One thread produces data, another consumes it.

**Concepts practiced**: `mpsc::channel`, ownership transfer, `Send`

**Requirements**:
- Producer thread generates 100 messages
- Consumer thread processes and prints each message
- Use `std::sync::mpsc::channel`

**Architecture**:
```
Producer Thread ──▶ mpsc::channel ──▶ Consumer Thread
```

**Extensions**: Add multiple producers. Add a bounded channel.

---

## Project 3: Async Producer/Consumer with Tokio

**Goal**: Rewrite Project 2 using `tokio::spawn` and `tokio::sync::mpsc`.

**Concepts practiced**: `async`, `.await`, `tokio::spawn`, `async move`, Tokio channels

**Requirements**:
- Async producer task sends market data events
- Async consumer task processes events
- Use bounded channel with backpressure
- Graceful shutdown via `CancellationToken`

**Architecture**:
```
tokio::spawn(producer) ──▶ mpsc::channel(100) ──▶ tokio::spawn(consumer)
                                                        │
CancellationToken ──────────────────────────────────────┘
```

---

## Project 4: Tokio Worker Pool

**Goal**: Process tasks concurrently across multiple worker tasks.

**Concepts practiced**: Worker pool pattern, channels, `Arc<Mutex<Receiver>>`

**Requirements**:
- 4 worker tasks consuming from a shared queue
- 100 tasks distributed among workers
- Each task takes 10-100ms (simulated)
- Print which worker processes which task

---

## Project 5: Axum REST API

**Goal**: Build a REST API for managing trading orders.

**Concepts practiced**: Axum, routing, extractors, JSON, shared state, error handling

**Requirements**:
- `POST /orders` — create an order
- `GET /orders` — list all orders
- `GET /orders/:id` — get specific order
- `DELETE /orders/:id` — cancel an order
- In-memory storage using `Arc<RwLock<HashMap>>`
- Custom error handling

**Folder structure**:
```
src/
├── main.rs
├── handlers.rs
├── models.rs
├── state.rs
└── errors.rs
```

---

## Project 6: Axum + PostgreSQL

**Goal**: Add PostgreSQL persistence to Project 5.

**Concepts practiced**: `sqlx`, connection pooling, async database queries, migrations

**Requirements**:
- Replace in-memory storage with PostgreSQL
- Use `sqlx::PgPool` as shared state
- Write SQL migrations for the orders table
- Handle database errors properly

---

## Project 7: WebSocket Echo Server

**Goal**: Build a WebSocket server that echoes messages back.

**Concepts practiced**: WebSocket with Axum, async bidirectional communication

**Requirements**:
- Axum WebSocket upgrade endpoint
- Echo received messages back
- Handle multiple concurrent connections
- Graceful disconnection handling

**Extensions**: Add a broadcast feature — messages from one client go to all clients.

---

## Project 8: Solana WebSocket Event Listener

**Goal**: Subscribe to Solana account changes via WebSocket.

**Concepts practiced**: Solana SDK, WebSocket subscriptions, async streams, event processing

**Requirements**:
- Connect to Solana WebSocket endpoint
- Subscribe to account updates for a specific program
- Parse and log account data changes
- Handle reconnection on disconnect
- Graceful shutdown

---

## Project 9: Solana Account Monitoring Service

**Goal**: Monitor multiple Solana accounts and track state changes.

**Concepts practiced**: Multiple subscriptions, shared state, channels, tracing

**Requirements**:
- Monitor multiple token accounts
- Track balance changes in a shared state (`Arc<RwLock<HashMap>>`)
- Expose current state via HTTP API (Axum)
- Log all changes with tracing
- Alert (print) when balance drops below threshold

**Architecture**:
```
WS Subscription 1 ─┐
WS Subscription 2 ─┤──▶ Event Channel ──▶ State Manager ──▶ Arc<RwLock<State>>
WS Subscription 3 ─┘                                              ▲
                                                                   │
HTTP API (Axum) ───────────────────────────────────────────────────┘
```

---

## Project 10: Async Solana Trading Engine Prototype

**Goal**: Build a simplified trading engine that monitors prices and submits transactions.

**Concepts practiced**: Everything from all chapters combined

**Requirements**:
- WebSocket listener for market data
- Event processing pipeline with channels
- Simple strategy (buy below threshold, sell above threshold)
- Transaction building and submission
- Position tracking in shared state
- HTTP API for status and manual control
- Structured logging with tracing
- Graceful shutdown
- Error handling with retries

**Architecture**:
```
src/
├── main.rs                 ← Bootstrap
├── config.rs               ← Configuration
├── state.rs                ← Shared state
├── ws_listener.rs          ← WebSocket market data
├── event_processor.rs      ← Parse and route events
├── strategy.rs             ← Trading logic
├── transaction.rs          ← Build and submit transactions
├── api/
│   ├── mod.rs              ← Axum router
│   ├── handlers.rs         ← HTTP handlers
│   └── models.rs           ← API request/response types
├── telemetry.rs            ← Tracing setup
└── errors.rs               ← Error types
```

**Concepts from each chapter used**:
| Chapter | Concept used |
|---------|-------------|
| 1-3 | Ownership, borrowing for data flow |
| 4 | Traits for strategy interface |
| 5 | Arc, Mutex for shared state |
| 6-7 | Understanding thread pool underneath |
| 8 | Send + 'static for spawned tasks |
| 9 | RwLock for read-heavy state |
| 10 | Channels between components |
| 11-14 | Async/await everywhere |
| 15 | Tokio mpsc, oneshot for request/response |
| 16 | tokio::sync::Mutex vs std::sync::Mutex |
| 17 | spawn_blocking for CPU work |
| 18 | anyhow for error handling |
| 19 | select! for shutdown + events |
| 26 | Axum HTTP API |
| 30 | Graceful shutdown |
| 31 | Tracing |

---

> **Next**: [Chapter 38: Interview Preparation](./38_interview_preparation.md)

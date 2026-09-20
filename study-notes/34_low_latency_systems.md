# Chapter 34: Low-Latency System Concepts

> **Connection to the bigger picture**: On-chain trading systems compete on speed. Understanding latency sources and optimization techniques is the difference between profitable and unprofitable trading. Async Rust gives you the tools, but you need to use them correctly.

---

## What Low Latency Means

Latency = time from event to action.

```
Market event occurs on Solana
       │
       ├── Network latency: event travels to your server (~5-50ms)
       ├── Deserialization: parse the event data (~μs)
       ├── Strategy computation: decide what to do (~μs-ms)
       ├── Transaction building: construct the transaction (~μs)
       ├── Serialization: encode for transmission (~μs)
       ├── Network latency: send to Solana node (~5-50ms)
       └── Confirmation: transaction lands on-chain (~400ms per slot)
       │
       Total: ~500ms-1s (dominated by network + confirmation)
```

---

## What You Can and Can't Optimize

| Factor | Optimizable? | How |
|--------|-------------|-----|
| **Network latency** | Partially | Co-locate with validators, use multiple nodes |
| **Deserialization** | Yes | Zero-copy parsing, efficient formats |
| **Strategy computation** | Yes | Efficient algorithms, avoid allocations |
| **Lock contention** | Yes | Lock-free data structures, minimize lock scope |
| **Memory allocation** | Yes | Pre-allocate, reuse buffers, arena allocators |
| **Channel overhead** | Yes | Choose appropriate channel types |
| **Async task overhead** | Minimal | Already very low (~ns switching) |
| **Solana slot time** | No | ~400ms per slot is fixed |
| **Transaction confirmation** | No | Depends on network and validator |

---

## CPU-Bound vs I/O-Bound

```
I/O-Bound (most of your system):
├── WebSocket connections     → Async tasks
├── HTTP RPC calls            → Async tasks
├── Database queries          → Async tasks
└── Network communication     → Async tasks

CPU-Bound (small but critical):
├── Signature verification    → spawn_blocking
├── Strategy calculations     → spawn_blocking or dedicated thread
├── Data serialization        → Inline if fast, spawn_blocking if slow
└── Cryptographic operations  → spawn_blocking
```

---

## Key Optimization Strategies

### 1. Minimize allocations

```rust
// ❌ Allocates a new String every time
fn format_key(symbol: &str, exchange: &str) -> String {
    format!("{}:{}", symbol, exchange)
}

// ✅ Reuse a buffer
struct KeyBuffer {
    buffer: String,
}

impl KeyBuffer {
    fn format(&mut self, symbol: &str, exchange: &str) -> &str {
        self.buffer.clear();
        self.buffer.push_str(symbol);
        self.buffer.push(':');
        self.buffer.push_str(exchange);
        &self.buffer
    }
}
```

### 2. Minimize lock scope

```rust
// ❌ Lock held during entire computation
let mut state = state.lock().await;
let result = expensive_computation(&state.data);
state.result = result;

// ✅ Lock only for data access
let data = {
    let state = state.lock().await;
    state.data.clone()
};
let result = expensive_computation(&data);
{
    let mut state = state.lock().await;
    state.result = result;
}
```

### 3. Connection reuse

```rust
// ❌ New connection per request
async fn fetch_price() -> f64 {
    let client = reqwest::Client::new();  // New connection!
    // ...
}

// ✅ Reuse client (connection pooling built-in)
struct PriceService {
    client: reqwest::Client,  // Created once, reused
}
```

### 4. Batch operations

```rust
// ❌ One RPC call per account
for account in accounts {
    let balance = client.get_balance(&account).await?;
}

// ✅ Batch RPC call
let balances = client.get_multiple_accounts(&accounts).await?;
```

---

## Important Tradeoffs

> **Async Rust does NOT automatically make your system low-latency.** It makes your system efficiently concurrent. Low latency requires:
> 
> 1. Minimizing the critical path (network + computation)
> 2. Reducing allocations and copies
> 3. Avoiding lock contention
> 4. Using appropriate data structures
> 5. Co-locating with infrastructure
> 6. Profiling and measuring

---

> **Next**: [Chapter 35: Jito / MEV Conceptual Foundation](./35_jito_mev.md)

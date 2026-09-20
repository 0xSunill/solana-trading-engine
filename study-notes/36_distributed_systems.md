# Chapter 36: Distributed System Basics

> **Connection to the bigger picture**: Your Solana backend is a distributed system — it communicates with Solana nodes, databases, and external APIs over the network. Understanding networking fundamentals is essential for debugging performance issues and designing reliable systems.

---

## Networking Fundamentals

### TCP vs UDP

| | TCP | UDP |
|---|---|---|
| **Connection** | Connection-oriented (handshake) | Connectionless |
| **Reliability** | Guaranteed delivery, ordering | Best effort, may lose packets |
| **Speed** | Slower (acknowledgments) | Faster (no overhead) |
| **Use case** | HTTP, WebSocket, database | DNS, video streaming, gaming |
| **Solana use** | RPC, WebSocket subscriptions | Gossip protocol (validators) |

### HTTP

HTTP is the protocol for your REST APIs and Solana RPC calls.

```
Client                          Server
  │                                │
  │──── GET /api/orders ──────────▶│
  │                                │
  │◀──── 200 OK + JSON body ──────│
  │                                │
```

### WebSocket

WebSocket provides full-duplex (bidirectional) communication over a single TCP connection.

```
Client                          Server
  │                                │
  │──── HTTP Upgrade Request ─────▶│  (WebSocket handshake)
  │◀──── 101 Switching Protocols ──│
  │                                │
  │◀──── message ─────────────────│  (server pushes data)
  │◀──── message ─────────────────│
  │──── message ──────────────────▶│  (client sends data)
  │◀──── message ─────────────────│
  │                                │
```

Ideal for Solana subscriptions — the server pushes events as they happen.

### RPC (Remote Procedure Call)

RPC makes calling functions on a remote server look like local function calls.

```rust
// Solana uses JSON-RPC:
// Request:
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getBalance",
    "params": ["wallet_address"]
}

// Response:
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "value": 1000000000
    }
}
```

---

## Key Distributed System Concepts

### Serialization / Deserialization

Converting data structures to/from bytes for network transmission.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Order {
    id: u64,
    symbol: String,
    price: f64,
}

// Serialize to JSON
let json = serde_json::to_string(&order)?;  // → {"id":1,"symbol":"SOL","price":150.0}

// Deserialize from JSON
let order: Order = serde_json::from_str(&json)?;

// Binary formats (faster):
// - bincode: Rust-native binary format
// - borsh: Solana's preferred format
// - protobuf: cross-language binary format
```

### Retries and Timeouts

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_retry(url: &str, max_retries: u32) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    
    for attempt in 0..max_retries {
        match timeout(Duration::from_secs(5), client.get(url).send()).await {
            Ok(Ok(response)) => return Ok(response.text().await?),
            Ok(Err(e)) => {
                tracing::warn!(attempt, error = %e, "Request failed, retrying");
            }
            Err(_) => {
                tracing::warn!(attempt, "Request timed out, retrying");
            }
        }
        
        // Exponential backoff
        tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
    }
    
    anyhow::bail!("All {} retries failed", max_retries)
}
```

### Backpressure

When producers are faster than consumers:

```
Without backpressure:
  Producer: 1000/s ──▶ [unbounded queue grows...] ──▶ Consumer: 100/s
  Memory usage: grows forever → OOM crash

With backpressure:
  Producer: 1000/s ──▶ [bounded channel, cap=100] ──▶ Consumer: 100/s
  Producer blocks when full → slows to 100/s → stable
```

### Idempotency

An operation is **idempotent** if calling it multiple times has the same effect as calling it once.

```rust
// NOT idempotent: each call adds to balance
async fn add_funds(amount: f64) { /* balance += amount */ }

// Idempotent: uses a unique ID to prevent duplicate processing
async fn add_funds_idempotent(request_id: Uuid, amount: f64) {
    if already_processed(request_id) { return; }
    // Process and record request_id as processed
}
```

Essential for retries — if a transaction submission times out, you don't know if it succeeded. With idempotency, retrying is safe.

---

## Things to Remember

1. **TCP** for reliable communication (HTTP, WebSocket). **UDP** for speed.
2. **WebSocket** for server-push events (Solana subscriptions).
3. **JSON-RPC** is Solana's communication protocol.
4. **Always set timeouts** — network requests can hang forever.
5. **Implement retries with exponential backoff** — transient failures are normal.
6. **Use bounded queues** for backpressure — prevent memory exhaustion.
7. **Design for idempotency** — retries are safer when operations are idempotent.
8. **Prefer binary serialization** (borsh, bincode) for performance-critical paths.

---

> **Next**: [Chapter 37: Practical Projects](./37_practical_projects.md)

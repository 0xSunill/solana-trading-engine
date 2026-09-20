# Chapter 35: Jito / MEV Conceptual Foundation

> **Connection to the bigger picture**: MEV (Maximal Extractable Value) is a key concept in blockchain trading. Understanding how transaction ordering works on Solana, what Jito provides, and how this connects to your async Rust backend is essential for on-chain trading.

---

## What Is MEV?

**MEV** = Maximal Extractable Value — the profit that can be gained by reordering, including, or excluding transactions within a block.

### How transactions are ordered

```
Users submit transactions
        │
        ▼
┌──────────────────┐
│ Transaction Pool  │  ← All pending transactions
│ (Mempool)         │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Block Producer   │  ← Validator building the next block
│ (Validator)      │
│                  │
│ Decides order:   │
│ 1. Tx with highest priority fee
│ 2. Tx from preferred sources
│ 3. Bundled transactions (Jito)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Block            │  ← Finalized order of transactions
│ [Tx1, Tx2, Tx3]  │
└──────────────────┘
```

### Common MEV strategies

| Strategy | What it does |
|----------|-------------|
| **Arbitrage** | Buy low on DEX A, sell high on DEX B — must execute atomically |
| **Liquidation** | Liquidate undercollateralized loans before others |
| **Sandwich** | Front-run and back-run a large trade to profit from price impact |
| **Backrunning** | Execute immediately after a specific transaction |

---

## Jito on Solana

Jito provides infrastructure for MEV on Solana:

### Bundles

A **bundle** is a group of transactions that must be executed together, in order, atomically.

```
Bundle = [Tx1, Tx2, Tx3]

Guarantees:
1. All transactions execute OR none do (atomic)
2. They execute in the specified order (Tx1 → Tx2 → Tx3)
3. No other transactions are inserted between them
```

### Block Engine

```
Your Trading Bot
      │
      │ Send bundle
      ▼
┌──────────────────┐
│ Jito Block       │  ← Receives bundles from searchers
│ Engine           │
│                  │
│ Evaluates:       │
│ - Is bundle      │
│   profitable?    │
│ - Does it        │
│   conflict?      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Jito-Enabled     │  ← Validator includes profitable bundles
│ Validator        │
└──────────────────┘
```

### Priority Fees

Transactions include **priority fees** (tips) to incentivize validators to include them sooner:

```rust
// Higher priority fee → more likely to be included quickly
let instruction = ComputeBudgetInstruction::set_compute_unit_price(
    50_000  // microlamports per compute unit
);
```

---

## How This Connects to Async Rust

Your trading bot architecture:

```
┌─────────────────────────────────────────────────────────┐
│ Async Rust Trading Bot                                  │
│                                                         │
│ ┌────────────────┐                                     │
│ │ Market Data    │ ← WebSocket subscription (async)    │
│ │ Listener       │                                     │
│ └───────┬────────┘                                     │
│         │ mpsc channel                                  │
│         ▼                                               │
│ ┌────────────────┐                                     │
│ │ Strategy       │ ← Detect opportunity (compute)      │
│ │ Engine         │                                     │
│ └───────┬────────┘                                     │
│         │ mpsc channel                                  │
│         ▼                                               │
│ ┌────────────────┐                                     │
│ │ Bundle Builder │ ← Build Jito bundle                 │
│ │                │                                     │
│ └───────┬────────┘                                     │
│         │                                               │
│         ▼                                               │
│ ┌────────────────┐                                     │
│ │ Jito Submitter │ ← Send to block engine (async HTTP) │
│ │                │                                     │
│ └───────┬────────┘                                     │
│         │                                               │
│         ▼                                               │
│ ┌────────────────┐                                     │
│ │ Confirmation   │ ← Monitor transaction (async poll)  │
│ │ Monitor        │                                     │
│ └────────────────┘                                     │
│                                                         │
│ Concurrency features used:                              │
│ ├── tokio::spawn for each component                     │
│ ├── mpsc channels between components                    │
│ ├── Arc<RwLock<State>> for shared market data           │
│ ├── select! for timeouts and shutdown                   │
│ └── spawn_blocking for CPU-intensive strategy           │
└─────────────────────────────────────────────────────────┘
```

### Why latency matters for MEV

```
Opportunity detected on Solana
        │
        ├── Your bot: 10ms to detect + 5ms to submit = 15ms
        │
        ├── Competitor: 8ms to detect + 3ms to submit = 11ms
        │
        └── Competitor wins — their bundle lands first
```

Every millisecond matters. This is why:
- Async Rust (no GC pauses) > Go/Java
- Pre-allocated buffers > dynamic allocation
- Connection reuse > new connections
- Co-location > remote servers
- Direct Jito connection > public RPC

---

## Things to Remember

1. **MEV = profit from transaction ordering** — it's the core of on-chain trading.
2. **Bundles** guarantee atomic, ordered execution.
3. **Priority fees** influence transaction inclusion speed.
4. **Latency is competitive** — faster detection + submission = more profit.
5. **Async Rust is ideal** — no GC pauses, efficient I/O, low overhead.
6. **Your architecture uses every async concept** — spawn, channels, select, Arc, shutdown.

---

> **Next**: [Chapter 36: Distributed System Basics](./36_distributed_systems.md)

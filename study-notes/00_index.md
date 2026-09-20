# 🦀 Rust, Concurrency & Async Rust — Complete Study Notes

## Goal: Become a Solana / On-Chain Trading Backend Engineer

These notes cover everything from Rust foundations through advanced async patterns to Solana-specific backend architecture. Each file is a deep-dive into its topic — not a summary, but real teaching material.

---

## How to Use These Notes

1. **Read in order** — each file builds on the previous ones
2. **Run every code example** — type them out, don't just read
3. **Try the "wrong code" examples first** — see the compiler errors yourself
4. **Do the exercises** — they cement understanding
5. **Review the cheat sheets** for quick revision before interviews

---

## Table of Contents

### Part 1 — Rust Foundations Required for Concurrency

| # | File | Topic |
|---|------|-------|
| 1 | [01_ownership.md](./01_ownership.md) | Ownership, Move Semantics, Stack vs Heap |
| 2 | [02_borrowing.md](./02_borrowing.md) | Borrowing, &T, &mut T, Borrow Checker |
| 3 | [03_references_and_lifetimes.md](./03_references_and_lifetimes.md) | References, Lifetimes, 'static, Elision |
| 4 | [04_traits.md](./04_traits.md) | Traits, Generics, Send, Sync, Future, Dispatch |
| 5 | [05_smart_pointers.md](./05_smart_pointers.md) | Box, Rc, Arc, RefCell, Mutex, RwLock |

### Part 2 — Rust Concurrency

| # | File | Topic |
|---|------|-------|
| 6 | [06_concurrency_vs_parallelism.md](./06_concurrency_vs_parallelism.md) | Concurrency, Parallelism, OS Scheduling |
| 7 | [07_os_threads.md](./07_os_threads.md) | std::thread, spawn, move, JoinHandle |

### Part 3 — Send and Sync

| # | File | Topic |
|---|------|-------|
| 8 | [08_send_and_sync.md](./08_send_and_sync.md) | Send, Sync, Marker Traits, Thread Safety |

### Part 4 — Shared State

| # | File | Topic |
|---|------|-------|
| 9 | [09_shared_state.md](./09_shared_state.md) | Mutex, RwLock, Arc, Shared Ownership |

### Part 5 — Channels and Message Passing

| # | File | Topic |
|---|------|-------|
| 10 | [10_channels_and_message_passing.md](./10_channels_and_message_passing.md) | mpsc, Sender, Receiver, Ownership Transfer |

### Part 6–9 — Async Rust Fundamentals

| # | File | Topic |
|---|------|-------|
| 11 | [11_async_fundamentals.md](./11_async_fundamentals.md) | Why Async Exists, Threads vs Tasks |
| 12 | [12_futures.md](./12_futures.md) | Future trait, Poll, Waker, .await |
| 13 | [13_tokio.md](./13_tokio.md) | Tokio Runtime, tokio::spawn, Scheduler |
| 14 | [14_async_move_ownership_lifetimes.md](./14_async_move_ownership_lifetimes.md) | async move, Capturing, 'static |

### Part 10–13 — Async Patterns

| # | File | Topic |
|---|------|-------|
| 15 | [15_tokio_channels.md](./15_tokio_channels.md) | mpsc, oneshot, broadcast, watch |
| 16 | [16_async_mutex_shared_state.md](./16_async_mutex_shared_state.md) | tokio::sync::Mutex vs std::sync::Mutex |
| 17 | [17_blocking_vs_nonblocking.md](./17_blocking_vs_nonblocking.md) | Blocking, spawn_blocking, CPU-bound work |
| 18 | [18_async_error_handling.md](./18_async_error_handling.md) | Result + Async, anyhow, thiserror |

### Part 14–17 — Advanced Async

| # | File | Topic |
|---|------|-------|
| 19 | [19_select_and_concurrent_tasks.md](./19_select_and_concurrent_tasks.md) | tokio::select!, Cancellation, Racing |
| 20 | [20_joining_tasks.md](./20_joining_tasks.md) | join!, try_join!, Concurrent Execution |
| 21 | [21_streams.md](./21_streams.md) | Stream, StreamExt, Async Iteration |
| 22 | [22_async_traits.md](./22_async_traits.md) | Async Trait Methods, Object Safety |

### Part 18 — Compiler Errors

| # | File | Topic |
|---|------|-------|
| 23 | [23_async_compiler_errors.md](./23_async_compiler_errors.md) | Common Errors, Diagnosis, Solutions |

### Part 19–20 — Design Patterns & Backend

| # | File | Topic |
|---|------|-------|
| 24 | [24_concurrency_design_patterns.md](./24_concurrency_design_patterns.md) | Producer/Consumer, Worker Pool, Actor Model |
| 25 | [25_async_backend_development.md](./25_async_backend_development.md) | HTTP, WebSockets, Database, Redis |

### Part 21–24 — Axum & Backend Architecture

| # | File | Topic |
|---|------|-------|
| 26 | [26_axum.md](./26_axum.md) | Axum Framework, Routing, Handlers, State |
| 27 | [27_backend_architecture.md](./27_backend_architecture.md) | Production Structure, Layers, DI |
| 28 | [28_tokio_axum_database.md](./28_tokio_axum_database.md) | Realistic Backend Example |
| 29 | [29_concurrency_safety_backend.md](./29_concurrency_safety_backend.md) | Race Conditions, Deadlocks, Backpressure |

### Part 25–26 — Production Concerns

| # | File | Topic |
|---|------|-------|
| 30 | [30_graceful_shutdown.md](./30_graceful_shutdown.md) | Shutdown Signals, CancellationToken, Draining |
| 31 | [31_tracing_debugging.md](./31_tracing_debugging.md) | tracing, Spans, Structured Logging |

### Part 27–30 — Solana & Trading Systems

| # | File | Topic |
|---|------|-------|
| 32 | [32_solana_connection.md](./32_solana_connection.md) | Solana RPC, WebSocket, Async Architecture |
| 33 | [33_realtime_market_data.md](./33_realtime_market_data.md) | Event Streams, State Updates, Strategy |
| 34 | [34_low_latency_systems.md](./34_low_latency_systems.md) | Latency, Throughput, Optimization |
| 35 | [35_jito_mev.md](./35_jito_mev.md) | MEV, Bundles, Priority Fees |

### Part 31 — Distributed Systems

| # | File | Topic |
|---|------|-------|
| 36 | [36_distributed_systems.md](./36_distributed_systems.md) | Networking, TCP, HTTP, Serialization |

### Part 32–35 — Projects, Interview Prep, Reference

| # | File | Topic |
|---|------|-------|
| 37 | [37_practical_projects.md](./37_practical_projects.md) | 10 Progressive Projects |
| 38 | [38_interview_preparation.md](./38_interview_preparation.md) | Questions & Answers by Topic |
| 39 | [39_cheat_sheets.md](./39_cheat_sheets.md) | Quick Reference Sheets |
| 40 | [40_final_mental_model.md](./40_final_mental_model.md) | Complete Connected Mental Model |

---

## The Big Picture

```
Rust Ownership System
       │
       ├── Move semantics → Thread safety (Send)
       ├── Borrowing rules → No data races at compile time
       └── Lifetimes → Safe references across async boundaries
              │
              ↓
       Concurrency Primitives
       │
       ├── Threads (std::thread)
       ├── Arc / Mutex (shared state)
       └── Channels (message passing)
              │
              ↓
       Async Rust
       │
       ├── Future / Poll / Waker
       ├── Tokio runtime
       ├── async/await
       └── Async channels / select / join
              │
              ↓
       Backend Development
       │
       ├── Axum (HTTP/WS framework)
       ├── Database pools
       ├── Error handling
       └── Graceful shutdown
              │
              ↓
       Solana Trading Systems
       │
       ├── RPC / WebSocket connections
       ├── Real-time event processing
       ├── Low-latency transaction submission
       └── MEV / Jito integration
```

> **Every topic in these notes builds toward the bottom of this diagram.**
> Nothing is taught in isolation — it all connects.

---

*Last updated: September 2026*
*Author's goal: Solana/On-chain Trading Backend Engineer*

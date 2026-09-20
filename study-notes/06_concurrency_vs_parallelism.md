# Chapter 6: Concurrency vs Parallelism

> **Connection to the bigger picture**: Before diving into threads, async tasks, and runtime internals, you need a crystal-clear understanding of concurrency and parallelism. Many engineers confuse these terms. In Solana trading systems, you need both — concurrency for handling thousands of WebSocket events, and parallelism for CPU-intensive calculations like order matching. Getting the distinction wrong leads to architectures that either waste resources or have unnecessary bottlenecks.

---

## Concurrency: Managing Multiple Things

**Concurrency** is about **structuring** a program to handle multiple tasks that can be in progress at the same time. The tasks don't necessarily run simultaneously — they just make progress by interleaving.

Think of a single chef in a kitchen:

```
Chef prepares:
├── Soup (starts boiling)         ← begins task 1
├── Salad (chop vegetables)       ← works on task 2 while soup boils
├── Soup (add seasoning)          ← returns to task 1
├── Salad (add dressing)          ← finishes task 2
└── Soup (serve)                  ← finishes task 1
```

One chef, multiple dishes, but only one thing is being done at any instant. The chef **interleaves** work on multiple dishes. This is concurrency.

---

## Parallelism: Doing Multiple Things Simultaneously

**Parallelism** is about **executing** multiple tasks at the exact same time. This requires multiple execution units (CPU cores).

Think of two chefs in a kitchen:

```
Chef 1:                        Chef 2:
├── Soup (starts boiling)      ├── Salad (chop vegetables)
├── Soup (add seasoning)       ├── Salad (add dressing)
└── Soup (serve)               └── Salad (serve)
```

Both chefs are working simultaneously. This is parallelism.

---

## The Critical Distinction

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  Concurrency = dealing with multiple things at once (STRUCTURE) │
│  Parallelism = doing multiple things at once (EXECUTION)        │
│                                                                 │
│  Concurrency is about DESIGN.                                   │
│  Parallelism is about HARDWARE.                                 │
│                                                                 │
│  You can have concurrency without parallelism.                  │
│  You can have parallelism without concurrency (SIMD).           │
│  Most real systems use both.                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| | Concurrency | Parallelism |
|---|---|---|
| **What** | Multiple tasks in progress | Multiple tasks executing simultaneously |
| **How many CPUs** | Can work with 1 | Requires 2+ |
| **Analogy** | One person juggling | Multiple people working |
| **Rust mechanism** | `async`/`await`, `select!`, channels | `thread::spawn`, `rayon`, multi-threaded runtime |
| **Good for** | I/O-bound work (network, disk) | CPU-bound work (computation) |
| **Example** | Handling 10,000 HTTP connections | Processing 10,000 images |

---

## CPU Cores and Threads

### What is a CPU core?

A CPU core is a physical execution unit that can run instructions. Modern computers have multiple cores:

```
CPU
├── Core 0  → runs one thread of instructions
├── Core 1  → runs another thread simultaneously
├── Core 2  → runs another
├── Core 3  → runs another
├── Core 4  → ...
├── Core 5
├── Core 6
└── Core 7
```

An 8-core CPU can run 8 threads truly simultaneously (in parallel).

### What is a thread?

A thread is a sequence of instructions that the OS can schedule to run on a CPU core.

```
Process (your program)
├── Thread 1 (main thread)
│   └── Stack, registers, instruction pointer
├── Thread 2
│   └── Stack, registers, instruction pointer
└── Thread 3
    └── Stack, registers, instruction pointer
    
All threads share: heap memory, file handles, code
```

**Key properties of OS threads:**
- Each thread has its own **stack** (typically 2-8 MB)
- All threads in a process share the same **heap** memory
- The OS **schedules** threads onto CPU cores
- Thread creation is relatively expensive (kernel system call)

### Context switching

When the OS has more threads than CPU cores, it uses **context switching** to give each thread a turn:

```
Time →
Core 0: [Thread A][Thread B][Thread A][Thread C][Thread A]
Core 1: [Thread D][Thread E][Thread D][Thread E][Thread D]

Context switch = saving Thread A's state, loading Thread B's state
```

Context switches are expensive:
- Save all CPU registers
- Flush CPU caches (cache misses after switch)
- Load the other thread's state
- Typically takes 1-10 microseconds

If you have 10,000 threads on 8 cores, the CPU spends most of its time context-switching rather than doing useful work.

---

## Multithreading vs Multitasking

**Multithreading**: Using multiple OS threads within a single process. Each thread can run on a different CPU core.

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        println!("Hello from a new thread!");
    });
    
    println!("Hello from the main thread!");
    handle.join().unwrap();
}
```

**Multitasking**: The OS running multiple processes (programs) concurrently, switching between them.

```
OS Scheduler
├── Process: Firefox      → has 50 threads
├── Process: VS Code      → has 30 threads
├── Process: Your Rust App → has 8 threads
└── ...
```

**Cooperative multitasking** (async Rust): Tasks voluntarily yield control, allowing other tasks to run. This is what `async`/`await` does.

```rust
// Cooperative: each task yields at .await
async fn task_a() {
    // do work
    some_io().await;  // "I'm waiting for I/O, someone else can run"
    // continue
}
```

**Preemptive multitasking** (OS threads): The OS forcibly takes control away from a thread (using timer interrupts) to run another.

---

## Real Examples

### I/O-Bound: Concurrent, not parallel

A web server handling 10,000 connections:

```
Most connections are WAITING for:
- Network data to arrive
- Database queries to complete
- External API responses

Only a few connections are actively computing at any instant.
```

For I/O-bound work, **concurrency** is what you need. You don't need 10,000 CPU cores — you need to efficiently manage 10,000 waiting tasks.

**Solution**: Async Rust (Tokio)
- 10,000 async tasks
- 4-8 OS threads (= number of cores)
- Tasks yield at `.await` points
- Tiny per-task overhead (a few hundred bytes)

### CPU-Bound: Parallel

Processing 10,000 images:

```
Each image requires:
- Decode (CPU-intensive)
- Resize (CPU-intensive)
- Encode (CPU-intensive)

No waiting involved — pure computation.
```

For CPU-bound work, **parallelism** is what you need. More cores = faster processing.

**Solution**: OS threads or Rayon
- 8 threads (one per core)
- Each thread processes images from a shared queue
- True parallel execution

### Solana Trading: Both

```
Concurrent (async):
├── WebSocket connection (waiting for data)
├── HTTP RPC calls (waiting for responses)
├── Timer events (waiting for time)
└── Channel communication (waiting for messages)

Parallel (threads/CPU):
├── Order matching algorithm (CPU-intensive)
├── Signature verification (CPU-intensive)
└── Data serialization (CPU-intensive)
```

A good Solana backend uses async for I/O and `spawn_blocking` or dedicated threads for CPU work.

---

## The Cost Comparison

| | OS Thread | Async Task |
|---|---|---|
| **Stack size** | 2-8 MB | ~few hundred bytes |
| **Creation time** | ~50-100 μs | ~few μs |
| **Context switch** | ~1-10 μs (kernel) | ~few ns (userspace) |
| **10,000 instances** | 20-80 GB of stack memory | ~few MB |
| **Scheduling** | OS kernel (preemptive) | Tokio runtime (cooperative) |
| **Good for** | CPU-bound work, blocking I/O | I/O-bound work, high concurrency |

This is why async Rust exists — to handle high concurrency without the enormous overhead of OS threads.

---

## Things to Remember

1. **Concurrency ≠ Parallelism.** Concurrency is about structure. Parallelism is about execution.
2. **Async Rust is concurrency.** It manages many tasks with few threads.
3. **Threads provide parallelism.** They execute code simultaneously on multiple cores.
4. **I/O-bound → async.** CPU-bound → threads.
5. **Context switching is expensive.** Async avoids it by using cooperative scheduling.
6. **Real systems use both.** Async for I/O, `spawn_blocking` for CPU work.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "Concurrency and parallelism are the same" | Concurrency is about managing multiple tasks. Parallelism is about executing them simultaneously. You can have concurrency on a single core. |
| "More threads = faster" | Beyond the number of CPU cores, adding threads adds overhead (context switching, memory). For I/O work, async tasks are more efficient. |
| "Async is always faster than threads" | For CPU-bound work, threads can be faster because they use all cores fully. Async is better for I/O-bound work with many waiting tasks. |
| "You must choose either threads or async" | Production systems use both. Tokio itself uses a thread pool internally. |

---

## Interview Questions

**Q: What is the difference between concurrency and parallelism?**

> Concurrency is about structuring a program to handle multiple tasks that can be in progress, potentially interleaving on a single core. Parallelism is about actually executing multiple tasks simultaneously, requiring multiple CPU cores. Concurrency is a design concern; parallelism is a hardware capability.

**Q: When would you use async tasks vs OS threads?**

> Async tasks for I/O-bound work (network, disk, database) where many tasks spend most of their time waiting. OS threads for CPU-bound work (computation, image processing) where tasks need continuous CPU access. For mixed workloads, use async with `spawn_blocking` for CPU work.

**Q: Why can't you just create 10,000 OS threads?**

> Each OS thread needs 2-8 MB of stack memory. 10,000 threads = 20-80 GB just for stacks. Plus, the kernel must context-switch between them, and with only 8 cores, most threads would be waiting. Async tasks use a few hundred bytes each and cooperatively yield, making them far more efficient for high concurrency.

---

> **Next**: [Chapter 7: OS Threads](./07_os_threads.md) — Creating and managing threads in Rust

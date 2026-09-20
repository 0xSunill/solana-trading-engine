# Chapter 12: Futures and `.await`

> **Connection to the bigger picture**: The `Future` trait is the foundation of all async Rust. Every `async fn`, every `async {}` block, every `.await` — they all work because of this trait. Understanding how futures work internally (polling, wakers, state machines) transforms async Rust from "magic that sometimes produces confusing errors" to "a system I understand and can debug."

---

## What Is a Future?

A future is a value that **will produce a result at some point in the future**. But that's the shallow definition. Let's go deeper.

### The `Future` trait

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),    // The future has completed, here's the value
    Pending,     // The future is not done yet, I'll notify you when to poll again
}
```

### Futures are lazy

This is critical. In Rust, creating a future **does nothing**. The future doesn't start executing until it's polled.

```rust
async fn fetch_data() -> String {
    println!("Fetching...");
    "data".to_string()
}

#[tokio::main]
async fn main() {
    let future = fetch_data();  // Creates a future — NOTHING HAPPENS YET
    println!("Future created");
    
    let data = future.await;    // NOW it executes (gets polled)
    println!("Got: {}", data);
}
```

Output:
```
Future created
Fetching...
Got: data
```

Compare with JavaScript where `async` functions start executing immediately — Rust futures are lazy.

### The polling model

Futures don't execute on their own. They need an **executor** (like Tokio) to drive them to completion.

```
┌──────────┐         ┌──────────────┐
│ Executor │ ──poll──▶ │   Future     │
│ (Tokio)  │         │              │
│          │ ◀───────│ Poll::Pending │
│          │         │              │
│ (does    │         │  (I/O not    │
│  other   │         │   ready yet) │
│  work)   │         │              │
│          │ ◀──wake──│ Waker fires │
│          │         │              │
│          │ ──poll──▶│              │
│          │         │              │
│          │ ◀───────│ Poll::Ready  │
│          │         │   (value)    │
└──────────┘         └──────────────┘
```

**Step by step:**

1. The executor calls `future.poll(cx)`.
2. The future does some work. If the result is available, it returns `Poll::Ready(value)`.
3. If the result is NOT available (e.g., waiting for network data), the future:
   a. Registers a **waker** from the `cx` (context) — "notify me when the I/O is ready"
   b. Returns `Poll::Pending`
4. The executor moves on to poll other futures.
5. When the I/O is ready, the waker fires, telling the executor to poll this future again.
6. On the next poll, the future completes and returns `Poll::Ready(value)`.

### Wakers — the notification mechanism

The `Waker` is how a future tells the executor "I'm ready to make progress, poll me again."

```rust
// Simplified — what happens inside a future:
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
    if self.data_is_ready() {
        Poll::Ready(self.take_data())
    } else {
        // Register the waker with the I/O system
        let waker = cx.waker().clone();
        self.io_driver.register_waker(waker);
        // When data arrives on the socket, the I/O driver calls waker.wake()
        // which tells the executor to poll this future again
        Poll::Pending
    }
}
```

The waker is essential for efficiency. Without it, the executor would have to constantly poll every future in a busy loop ("are you done yet? are you done yet?"). With wakers, the executor only polls futures that have new data to process.

---

## State Machine Transformation

When you write an `async fn`, the compiler transforms it into a state machine. This is the most important internal detail to understand.

### Your code:

```rust
async fn fetch_and_process() -> String {
    let response = make_request().await;    // Suspension point 1
    let data = parse_response(response).await;  // Suspension point 2
    format!("Processed: {}", data)
}
```

### What the compiler generates (conceptually):

```rust
enum FetchAndProcessState {
    Start,
    WaitingForRequest { future: MakeRequestFuture },
    WaitingForParse { future: ParseResponseFuture },
    Done,
}

impl Future for FetchAndProcess {
    type Output = String;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        loop {
            match self.state {
                State::Start => {
                    let future = make_request();
                    self.state = State::WaitingForRequest { future };
                }
                State::WaitingForRequest { ref mut future } => {
                    match future.poll(cx) {
                        Poll::Ready(response) => {
                            let future = parse_response(response);
                            self.state = State::WaitingForParse { future };
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                State::WaitingForParse { ref mut future } => {
                    match future.poll(cx) {
                        Poll::Ready(data) => {
                            self.state = State::Done;
                            return Poll::Ready(format!("Processed: {}", data));
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                State::Done => panic!("polled after completion"),
            }
        }
    }
}
```

**Key insight**: Each `.await` point becomes a state in the state machine. The state machine stores whatever data is needed to resume execution from that point. This is why:

1. **Futures have a known size at compile time** — the enum contains all possible states.
2. **Futures are values** — they're just structs/enums stored on the stack or heap.
3. **Resuming is cheap** — just match on the current state and continue.
4. **Data held across `.await` is part of the state** — this is why it must be `Send` for `tokio::spawn`.

---

## What `.await` Really Does

When Rust encounters `some_future.await`:

1. **It does NOT block the thread.** This is the most important thing.
2. It polls the inner future.
3. If the inner future returns `Poll::Ready(value)`, `.await` evaluates to `value` and execution continues.
4. If the inner future returns `Poll::Pending`, the **entire enclosing async function** returns `Poll::Pending` up to its caller.
5. The current task is suspended — the executor is free to run other tasks on this thread.
6. When the waker fires, the executor polls the enclosing future again, which resumes from the `.await` point.

### `.await` vs blocking

```rust
// ❌ BLOCKS THE THREAD — no other task can run
std::thread::sleep(std::time::Duration::from_secs(1));

// ✅ YIELDS THE TASK — other tasks run on this thread
tokio::time::sleep(std::time::Duration::from_secs(1)).await;
```

**Why `thread::sleep` is disastrous in async code:**

```
With thread::sleep(1s) on a 4-thread Tokio runtime:
  Thread 1: [task1: thread::sleep—blocks entire thread for 1s—]
  Thread 2: [task2: doing work]
  Thread 3: [task3: doing work]
  Thread 4: [task4: doing work]
  
  Tasks 5-10,000: waiting for Thread 1 to be available
  Throughput: reduced by 25%!

With tokio::time::sleep(1s):
  Thread 1: [task1: yields][task5][task6][task7]...[task1: resumes]
  Thread 2: [task2][task8][task9]...
  Thread 3: [task3][task10][task11]...
  Thread 4: [task4][task12][task13]...
  
  All tasks make progress. No thread is wasted.
```

---

## Async Functions Return Futures

```rust
// This async function:
async fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Is equivalent to:
fn greet(name: &str) -> impl Future<Output = String> + '_ {
    async move {
        format!("Hello, {}!", name)
    }
}
```

The return type is `impl Future<Output = String>` — an opaque type that implements `Future` and will produce a `String` when polled to completion.

---

## Composing Futures

Futures compose naturally:

```rust
async fn step_1() -> i32 {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    1
}

async fn step_2(input: i32) -> i32 {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    input + 1
}

async fn pipeline() -> i32 {
    let a = step_1().await;       // Wait for step 1
    let b = step_2(a).await;      // Then wait for step 2
    b
}

// pipeline() is itself a Future<Output = i32>
```

---

## Things to Remember

1. **Futures are lazy** — creating a future does nothing until it's polled.
2. **`.await` does NOT block** — it suspends the current task and lets the thread run other tasks.
3. **Futures are state machines** — the compiler transforms async code into enum states.
4. **Polling drives progress** — the executor calls `poll()`, the future does work or returns `Pending`.
5. **Wakers notify the executor** — "this future is ready to make progress."
6. **Data across `.await` is stored in the state machine** — this affects `Send` and size.
7. **Never use blocking operations in async** — they block the entire thread, starving other tasks.

---

## Common Misconceptions

| Misconception | Reality |
|---------------|---------|
| "`.await` blocks until the operation completes" | `.await` yields the current task. The thread runs other tasks while waiting. |
| "Futures execute when created" | Futures are lazy. They execute when polled (usually via `.await`). |
| "Async automatically makes things parallel" | Async provides concurrency, not parallelism. Tasks are interleaved on threads, not necessarily running simultaneously. |
| "Each async task gets its own thread" | Many tasks share a few threads. That's the whole point. |

---

## Interview Questions

**Q: How does the Rust `Future` trait work?**

> A Future has a `poll()` method that returns `Poll::Ready(value)` when complete or `Poll::Pending` when waiting. The executor calls `poll()`, and the future registers a `Waker` to notify the executor when it should be polled again. Futures are lazy — they only execute when polled.

**Q: What happens internally when you write `.await`?**

> The compiler transforms the async function into a state machine. Each `.await` point is a state transition. When `.await` is reached, the inner future is polled. If it returns `Pending`, the outer future also returns `Pending`, suspending the task. The thread is free to run other tasks. When the waker fires, the future is polled again and resumes from the saved state.

**Q: Why should you never call `std::thread::sleep()` in async code?**

> `thread::sleep()` blocks the OS thread, preventing it from running any other async tasks. On a Tokio runtime with 4 threads, blocking one thread reduces capacity by 25%. Use `tokio::time::sleep().await` instead — it yields the task, letting the thread run others.

---

> **Next**: [Chapter 13: Tokio Runtime](./13_tokio.md) — The engine that drives async Rust

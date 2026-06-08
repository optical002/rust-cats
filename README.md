# rust-cats

A Rust port of Scala's [cats](https://typelevel.org/cats/) and
[cats-effect](https://typelevel.org/cats-effect/) typeclass hierarchies, using
brand/witness types to emulate HKTs.

## Workspace layout

- `core/` (`rust-cats-core`) — `Functor`, `Apply`, `Applicative`, `FlatMap`,
  `Monad`, `Invariant`, `ApplicativeError`, `MonadError`, `Defer`, `Semigroup`,
  `Monoid`, plus instances for `Option`, `Result`, and `Eval`, syntax extension
  traits, and a `for_!` macro for for-comprehensions.
- `effect/` (`rust-cats-effect`) — `MonadCancel`, `Unique`, `Clock`, `Spawn`,
  `Concurrent` (`Ref`, `Deferred`), `Temporal`, `Sync`, `Async`. Includes an
  `IO` data type plus a **multi-threaded work-stealing scheduler** built on
  `crossbeam-deque`.
- `benchmark/` — benchmark binaries comparing our `IO` scheduler against `tokio`.

## Scheduler

The runtime spawns `num_cpus::get()` worker threads (override with
`RUST_CATS_WORKERS`). Each worker has its own `crossbeam-deque` FIFO local queue
plus access to a global `Injector` for cross-thread submission. The find-task
strategy is local-pop → injector-steal → sibling-stealers. Workers park on per-
worker `Mutex<bool> + Condvar` slots; `enqueue()` wakes one idle worker. A
dedicated timer thread runs the `sleep` wheel.

Fiber join, `Deferred::get`, and `Async::async_io` use a `ParkSlot` (a small
condvar wrapper): the calling thread blocks on it while other workers continue
to drain the work queue, and the completion callback signals it.

## Benchmark: prime sieve

Both binaries count primes in `[0, 1_000_000)` across 100 concurrent fibers
(10,000 numbers per fiber) and report wall time.

```
cargo run --release -p benchmark --bin prime_sieve_cats
cargo run --release -p benchmark --bin prime_sieve_tokio
```

### Results

Hardware: Linux x86_64, 24 cores. Each binary run 3 times.

| Runtime                                       | Avg time | Result        |
|-----------------------------------------------|---------:|---------------|
| `rust-cats-effect` (single-threaded, pre)     |    91 ms | 78498 primes  |
| `rust-cats-effect` (work-stealing, current)   |     8 ms | 78498 primes  |
| `tokio` (multi-threaded)                      |     8 ms | 78498 primes  |

The work-stealing scheduler delivers an **~11x speedup** over the original
single-threaded one and is **at parity with `tokio`** on this CPU-bound
workload. Theoretical parallel lower bound is `91 ms / 24 ≈ 3.8 ms`; both
schedulers land around 8 ms, with the remaining gap covering spawn / join /
steal overhead.

Raw run logs are in `benchmark/benchmarks.txt`.

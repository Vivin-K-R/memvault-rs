# memvault-rs

A safe Rust library implementing two foundational memory primitives from scratch:
a **Bump Arena Allocator** and a **Ring Buffer**. Built as a capstone project for a Secure
Coding course, demonstrating the core Rust philosophy of using `unsafe` internally while
exposing a completely safe, ergonomic public API.

---

## What's Inside

### Bump Arena Allocator
A contiguous memory region where allocation is performed by bumping an offset forward.
All allocations are freed at once by resetting the arena.

- Constant-time allocation (pointer arithmetic only)
- Excellent cache locality
- No per-allocation free cost
- Safe typed API restricted to `Copy` types — destructor bugs impossible by construction
- Only 2 small `unsafe` blocks, both fully justified

### Ring Buffer (SPSC)
A fixed-capacity circular queue ideal for streaming and real-time pipelines.

- Bounded, predictable memory usage
- O(1) push and pop
- Explicit backpressure via `Result` return types
- Zero `unsafe` — fully safe Rust

---

## Project Structure
```
memvault-rs/
├── src/
│   └── lib.rs          # Arena + RingBuffer implementation + tests
├── benches/
│   └── benchmark.rs    # Criterion benchmarks
└── Cargo.toml
```

---

## Building
```bash
# Build the library
cargo build

# Run all tests
cargo test

# Run benchmarks (generates HTML report)
cargo bench
```

---

## Test Results
```
test tests::arena_basic          ... ok
test tests::arena_out_of_memory  ... ok
test tests::ring_basic           ... ok
test tests::ring_invariants      ... ok
test tests::ring_wraparound      ... ok

test result: ok. 5 passed; 0 failed
```

---

## Benchmark Results

| Benchmark | Time |
|---|---|
| Heap allocation (1000 iterations) | 1.07 µs |
| Arena allocation (1000 iterations) | 3.07 µs |
| RingBuffer (100,000 push/pop) | 84.7 µs |
| VecDeque (100,000 push/pop) | 289.5 µs |

**RingBuffer beats VecDeque by 3.4x** due to fixed memory, zero reallocation,
and pure index arithmetic.

Full HTML benchmark report generated at `target/criterion/report/index.html`
after running `cargo bench`.

---

## Safety Philosophy

This library demonstrates the essence of systems Rust — use low-level power
internally, expose safe well-specified abstractions externally.

| Principle | Implementation |
|---|---|
| Unsafe isolation | All `unsafe` confined to 2 small auditable blocks |
| Type system as security | `T: Copy` constraint makes destructor bugs impossible |
| Explicit failure | `Option` and `Result` force callers to handle errors |
| Invariant documentation | Safety proof written before each `unsafe` call |
| Verified correctness | Boundary condition tests catch wraparound and overflow bugs |

---

## Key Concepts

- **Bump allocation** — amortized allocation by moving an offset forward
- **Alignment math** — manual alignment matching what C compilers do automatically
- **Ring buffer wraparound** — circular indexing with unambiguous full/empty state
- **`NonNull<T>`** — guaranteed non-null pointer wrapper replacing raw `*mut T`
- **Criterion benchmarking** — statistical performance measurement with HTML reports

---


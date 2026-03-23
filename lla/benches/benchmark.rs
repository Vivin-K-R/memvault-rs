use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lla::{Arena, RingBuffer};
use std::collections::VecDeque;

// ── Arena vs Heap ────────────────────────────────────────────────

fn bench_heap_alloc(n: usize) -> usize {
    let mut checksum = 0usize;
    for i in 0..n {
        let mut v = Vec::<u32>::with_capacity(16);
        for j in 0..16u32 {
            v.push((i as u32) ^ j);
        }
        checksum ^= v[0] as usize;
    }
    checksum
}

fn bench_arena_alloc(n: usize) -> usize {
    let mut a = Arena::with_capacity(n * 16 * 4 + 128);
    let mut checksum = 0usize;
    for i in 0..n {
        let xs = a.alloc_slice::<u32>(16).unwrap();
        for (j, x) in xs.iter_mut().enumerate() {
            *x = (i as u32) ^ (j as u32);
        }
        checksum ^= xs[0] as usize;
    }
    checksum
}

// ── RingBuffer vs VecDeque ───────────────────────────────────────

fn bench_ring(n: usize) -> u32 {
    let mut rb = RingBuffer::with_capacity(1024, 0u32);
    let mut acc = 0u32;
    for i in 0..n as u32 {
        if rb.push(i).is_ok() {
            if let Ok(x) = rb.pop() {
                acc ^= x;
            }
        }
    }
    acc
}

fn bench_vecdeque(n: usize) -> u32 {
    let mut q = VecDeque::<u32>::with_capacity(1024);
    let mut acc = 0u32;
    for i in 0..n as u32 {
        if q.len() < 1024 {
            q.push_back(i);
        }
        if let Some(x) = q.pop_front() {
            acc ^= x;
        }
    }
    acc
}

// ── Criterion Groups ─────────────────────────────────────────────

fn arena_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Allocation");
    group.bench_function("heap_alloc_1000", |b| {
        b.iter(|| bench_heap_alloc(black_box(1000)))
    });
    group.bench_function("arena_alloc_1000", |b| {
        b.iter(|| bench_arena_alloc(black_box(1000)))
    });
    group.finish();
}

fn ring_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Queue");
    group.bench_function("ringbuffer_100000", |b| {
        b.iter(|| bench_ring(black_box(100_000)))
    });
    group.bench_function("vecdeque_100000", |b| {
        b.iter(|| bench_vecdeque(black_box(100_000)))
    });
    group.finish();
}

criterion_group!(benches, arena_benchmark, ring_benchmark);
criterion_main!(benches);
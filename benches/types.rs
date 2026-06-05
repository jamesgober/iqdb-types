//! Benchmarks for the compute-bearing paths in `iqdb-types`.
//!
//! The crate is mostly trivial, inlined accessors; the two paths that do real
//! work are:
//!
//! - **`Vector::new`** — the empty + non-finite validation scan plus the
//!   shrink-to-`Box<[f32]>` on success (the ingest hot path).
//! - **`VectorId` `Display`** — lowercase-hex rendering of a byte key (used on
//!   logging / error-reporting paths, once per rendered hit).
//!
//! Run with `cargo bench`. Reports land in `target/criterion/`. Throughput is
//! reported per element / per byte, so cost should scale linearly.

use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use iqdb_types::{Vector, VectorId};

fn bench_vector_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_new");

    // Common embedding dimensionalities: small, MiniLM-ish, OpenAI-ish, large.
    for dim in [32usize, 128, 768, 1024] {
        let data: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.001).collect();

        group.throughput(Throughput::Elements(dim as u64));
        group.bench_with_input(BenchmarkId::from_parameter(dim), &data, |b, data| {
            // `Vector::new` consumes its input, so each iteration gets a fresh
            // clone via the batched setup — the clone is excluded from the
            // measured routine.
            b.iter_batched(
                || data.clone(),
                |owned| black_box(Vector::new(owned).unwrap()),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_vector_id_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_id_display");

    // Key sizes: UUID (16), SHA-256 (32), and a long opaque key (64).
    for len in [16usize, 32, 64] {
        let id = VectorId::try_from((0..len).map(|i| i as u8).collect::<Vec<u8>>())
            .expect("non-empty key");

        group.throughput(Throughput::Bytes(len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(len), &id, |b, id| {
            b.iter(|| black_box(id.to_string()));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_vector_new, bench_vector_id_display);
criterion_main!(benches);

//! Benchmark for the one measurable hot path in `iqdb-types`:
//! [`Vector::new`] validation (the empty + non-finite scan) plus the
//! shrink-to-`Box<[f32]>` on success.
//!
//! Run with: `cargo bench --bench vector_new`. Reports land in
//! `target/criterion/`. Throughput is reported per element, so the cost should
//! scale linearly with dimensionality.

use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use iqdb_types::Vector;

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

criterion_group!(benches, bench_vector_new);
criterion_main!(benches);

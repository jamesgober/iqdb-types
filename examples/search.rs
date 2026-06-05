//! Describing a similarity search and reading its results.
//!
//! Run with: `cargo run --example search`

use iqdb_types::{DistanceMetric, Filter, Hit, Metadata, SearchParams, Value, VectorId};

fn main() {
    // --- Tier 1: the common case in a single call -------------------------
    let simple = SearchParams::new(10, DistanceMetric::Cosine);
    println!("top-{} by {:?}", simple.k, simple.metric);

    // --- Tier 2: tune the optional knobs via struct-update ----------------
    let tuned = SearchParams {
        ef: Some(128), // candidate-list size for approximate indexes
        filter: Some(Filter::eq("published", Value::Bool(true))),
        ..SearchParams::new(10, DistanceMetric::Euclidean)
    };
    println!(
        "k={} metric={:?} ef={:?} filtered={}",
        tuned.k,
        tuned.metric,
        tuned.ef,
        tuned.filter.is_some(),
    );

    // --- Hits: what a search returns --------------------------------------
    let meta: Metadata = [("title".to_string(), Value::String("intro".to_string()))]
        .into_iter()
        .collect();
    let hit = Hit {
        metadata: Some(meta),
        ..Hit::new(VectorId::from(42u64), 0.0125)
    };
    println!(
        "hit id={} distance={} title={:?}",
        hit.id,
        hit.distance,
        hit.metadata.as_ref().and_then(|m| m.get("title")),
    );
}

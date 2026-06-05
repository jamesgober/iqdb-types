//! Consumer-simulation integration tests — the v0.4.0 ROADMAP gate.
//!
//! The roadmap gates the feature freeze on "at least two downstream iQDB crates
//! compile against this unchanged." Those crates are not part of this
//! repository, so instead we build minimal *working analogues* of the three
//! real consumers — `iqdb-distance`, an index crate (`iqdb-flat`-shaped), and
//! `iqdb-filter` — using **only** the public `iqdb-types` surface, and against
//! the **exact signatures those crates expose**:
//!
//! - `iqdb-distance` — `compute(metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>`
//! - `iqdb-index` — `insert(id: VectorId, vector: Arc<[f32]>, metadata: Option<Metadata>)`;
//!   `search(query: &[f32], params: &SearchParams) -> Result<Vec<Hit>>`; `delete(id: &VectorId)`
//! - `iqdb-filter` — evaluate a `Filter` tree against a record's `Metadata`
//!
//! If a real (if tiny) vector database can be assembled from these types alone
//! at those signatures, the vocabulary is sufficient and ergonomic for the
//! family. Anything awkward here is a design smell to fix before the 1.0 freeze.

#![allow(clippy::unwrap_used)]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use iqdb_types::{
    DistanceMetric, Filter, Hit, IqdbError, Metadata, Result, SearchParams, Value, Vector, VectorId,
};

// ---------------------------------------------------------------------------
// Stand-in for `iqdb-distance` — mirrors `dispatch::compute`'s signature.
// ---------------------------------------------------------------------------

/// Distance between two vectors under `metric` (smaller is nearer). Same shape
/// as `iqdb_distance::compute(metric, a, b)`: it takes `&[f32]`, not a wrapper.
fn compute(metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != b.len() {
        return Err(IqdbError::DimensionMismatch {
            expected: a.len(),
            found: b.len(),
        });
    }
    let dist = match metric {
        DistanceMetric::Euclidean => a
            .iter()
            .zip(b)
            .map(|(p, q)| (p - q).powi(2))
            .sum::<f32>()
            .sqrt(),
        DistanceMetric::Manhattan => a.iter().zip(b).map(|(p, q)| (p - q).abs()).sum(),
        DistanceMetric::DotProduct => -a.iter().zip(b).map(|(p, q)| p * q).sum::<f32>(),
        DistanceMetric::Cosine => {
            let dot: f32 = a.iter().zip(b).map(|(p, q)| p * q).sum();
            let na = a.iter().map(|p| p * p).sum::<f32>().sqrt();
            let nb = b.iter().map(|q| q * q).sum::<f32>().sqrt();
            if na == 0.0 || nb == 0.0 {
                return Err(IqdbError::InvalidVector);
            }
            1.0 - dot / (na * nb)
        }
        DistanceMetric::Hamming => a.iter().zip(b).filter(|(p, q)| p != q).count() as f32,
        // `DistanceMetric` is #[non_exhaustive], so a consumer in another crate
        // MUST write this arm. (Note: the real `iqdb-distance::compute` matches
        // the five variants WITHOUT a wildcard, so adding this arm is the one
        // edit that crate needs when merged against iqdb-types >= 0.3.)
        _ => return Err(IqdbError::InvalidMetric),
    };
    Ok(dist)
}

// ---------------------------------------------------------------------------
// Stand-in for `iqdb-filter` — evaluate a Filter tree (closed-world).
// ---------------------------------------------------------------------------

fn eval_filter(filter: &Filter, meta: Option<&Metadata>) -> bool {
    match filter {
        Filter::Eq { field, value } => leaf(meta, field, |v| v == value),
        Filter::Neq { field, value } => leaf(meta, field, |v| v != value),
        Filter::Lt { field, value } => leaf(meta, field, |v| value_lt(v, value)),
        Filter::Lte { field, value } => leaf(meta, field, |v| value_lt(v, value) || v == value),
        Filter::Gt { field, value } => leaf(meta, field, |v| value_lt(value, v)),
        Filter::Gte { field, value } => leaf(meta, field, |v| value_lt(value, v) || v == value),
        Filter::In { field, values } => leaf(meta, field, |v| values.iter().any(|x| x == v)),
        Filter::And(subs) => subs.iter().all(|f| eval_filter(f, meta)),
        Filter::Or(subs) => subs.iter().any(|f| eval_filter(f, meta)),
        Filter::Not(inner) => !eval_filter(inner, meta),
    }
}

/// A leaf comparison: absent field => `false` (closed-world).
fn leaf(meta: Option<&Metadata>, field: &str, pred: impl Fn(&Value) -> bool) -> bool {
    match meta.and_then(|m| m.get(field)) {
        Some(v) => pred(v),
        None => false,
    }
}

/// Type-aware "less than"; a type mismatch is `false` (closed-world). Proves
/// `Value` ordering is implementable from the public surface even though
/// `Value` is only `PartialEq` (it holds an `f64`).
fn value_lt(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x < y,
        (Value::Float(x), Value::Float(y)) => x < y,
        (Value::String(x), Value::String(y)) => x < y,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Stand-in for an index crate — mirrors the `IndexCore` trait shape:
// stores `Arc<[f32]>`, searches on `&[f32]`, deletes by `&VectorId`.
// ---------------------------------------------------------------------------

struct MiniIndex {
    dim: usize,
    metric: DistanceMetric,
    records: HashMap<VectorId, (Arc<[f32]>, Option<Metadata>)>,
}

impl MiniIndex {
    fn new(dim: usize, metric: DistanceMetric) -> Self {
        Self {
            dim,
            metric,
            records: HashMap::new(),
        }
    }

    fn insert(
        &mut self,
        id: VectorId,
        vector: Arc<[f32]>,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        if vector.len() != self.dim {
            return Err(IqdbError::DimensionMismatch {
                expected: self.dim,
                found: vector.len(),
            });
        }
        if self.records.contains_key(&id) {
            return Err(IqdbError::Duplicate);
        }
        let _ = self.records.insert(id, (vector, metadata));
        Ok(())
    }

    fn delete(&mut self, id: &VectorId) -> Result<()> {
        match self.records.remove(id) {
            Some(_) => Ok(()),
            None => Err(IqdbError::NotFound),
        }
    }

    fn search(&self, query: &[f32], params: &SearchParams) -> Result<Vec<Hit>> {
        if params.metric != self.metric {
            return Err(IqdbError::InvalidMetric);
        }
        if query.len() != self.dim {
            return Err(IqdbError::DimensionMismatch {
                expected: self.dim,
                found: query.len(),
            });
        }
        let mut hits: Vec<Hit> = Vec::new();
        for (id, (vector, meta)) in &self.records {
            if let Some(filter) = &params.filter {
                if !eval_filter(filter, meta.as_ref()) {
                    continue;
                }
            }
            let d = compute(self.metric, query, vector)?;
            hits.push(Hit {
                metadata: meta.clone(),
                ..Hit::new(id.clone(), d)
            });
        }
        hits.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });
        hits.truncate(params.k);
        Ok(hits)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn meta(pairs: &[(&str, Value)]) -> Metadata {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect()
}

/// The engine's real construction path: validate through `Vector::new`, then
/// hand the index a shared `Arc<[f32]>` (no copy of the data).
fn validated(data: Vec<f32>) -> Arc<[f32]> {
    Vector::new(data).unwrap().into_inner().into()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn distance_matches_known_values_for_every_metric() {
    let (a, b) = (vec![1.0_f32, 0.0], vec![0.0_f32, 1.0]);
    let approx = |x: f32, y: f32| (x - y).abs() < 1e-6;

    assert!(approx(
        compute(DistanceMetric::Euclidean, &a, &b).unwrap(),
        std::f32::consts::SQRT_2,
    ));
    assert!(approx(
        compute(DistanceMetric::Manhattan, &a, &b).unwrap(),
        2.0
    ));
    assert!(approx(
        compute(DistanceMetric::DotProduct, &a, &b).unwrap(),
        0.0
    ));
    assert!(approx(
        compute(DistanceMetric::Cosine, &a, &b).unwrap(),
        1.0
    ));
    assert!(approx(
        compute(DistanceMetric::Hamming, &a, &b).unwrap(),
        2.0
    ));
}

#[test]
fn distance_rejects_dimension_mismatch() {
    let err = compute(DistanceMetric::Euclidean, &[1.0, 2.0, 3.0], &[1.0, 2.0]).unwrap_err();
    assert_eq!(
        err,
        IqdbError::DimensionMismatch {
            expected: 3,
            found: 2
        }
    );
}

#[test]
fn index_insert_validates_dimension_and_rejects_duplicates() {
    let mut index = MiniIndex::new(2, DistanceMetric::Cosine);
    index
        .insert(VectorId::from(1u64), validated(vec![0.0, 1.0]), None)
        .unwrap();

    // Wrong dimensionality.
    let dim_err = index
        .insert(VectorId::from(2u64), validated(vec![0.0, 1.0, 2.0]), None)
        .unwrap_err();
    assert_eq!(
        dim_err,
        IqdbError::DimensionMismatch {
            expected: 2,
            found: 3
        }
    );

    // Duplicate id.
    let dup_err = index
        .insert(VectorId::from(1u64), validated(vec![1.0, 0.0]), None)
        .unwrap_err();
    assert_eq!(dup_err, IqdbError::Duplicate);
}

#[test]
fn index_delete_reports_not_found() {
    let mut index = MiniIndex::new(2, DistanceMetric::Cosine);
    index
        .insert(VectorId::from(1u64), validated(vec![0.0, 1.0]), None)
        .unwrap();

    assert!(index.delete(&VectorId::from(1u64)).is_ok());
    assert_eq!(
        index.delete(&VectorId::from(1u64)).unwrap_err(),
        IqdbError::NotFound
    );
}

#[test]
fn search_rejects_metric_and_dimension_mismatch() {
    let index = MiniIndex::new(2, DistanceMetric::Cosine);

    let wrong_metric = index
        .search(
            &[1.0, 0.0],
            &SearchParams::new(1, DistanceMetric::Euclidean),
        )
        .unwrap_err();
    assert_eq!(wrong_metric, IqdbError::InvalidMetric);

    let wrong_dim = index
        .search(
            &[1.0, 0.0, 0.0],
            &SearchParams::new(1, DistanceMetric::Cosine),
        )
        .unwrap_err();
    assert_eq!(
        wrong_dim,
        IqdbError::DimensionMismatch {
            expected: 2,
            found: 3
        }
    );
}

#[test]
fn end_to_end_filtered_top_k_search() {
    let mut index = MiniIndex::new(2, DistanceMetric::Cosine);
    index
        .insert(
            VectorId::from(1u64),
            validated(vec![1.0, 0.0]),
            Some(meta(&[
                ("published", Value::Bool(true)),
                ("year", Value::Int(2025)),
            ])),
        )
        .unwrap();
    index
        .insert(
            VectorId::from(2u64),
            validated(vec![0.9, 0.1]),
            Some(meta(&[
                ("published", Value::Bool(false)),
                ("year", Value::Int(2026)),
            ])),
        )
        .unwrap();
    index
        .insert(
            VectorId::from(3u64),
            validated(vec![0.8, 0.2]),
            Some(meta(&[
                ("published", Value::Bool(true)),
                ("year", Value::Int(2026)),
            ])),
        )
        .unwrap();

    let params = SearchParams {
        filter: Some(Filter::eq("published", Value::Bool(true))),
        ..SearchParams::new(2, DistanceMetric::Cosine)
    };
    let hits = index.search(&[1.0, 0.0], &params).unwrap();

    // Record 2 is filtered out; 1 and 3 remain, with 1 nearest, and metadata
    // flows back on the hit.
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, VectorId::U64(1));
    assert_eq!(hits[1].id, VectorId::U64(3));
    assert!(hits[0].distance <= hits[1].distance);
    assert_eq!(
        hits[0].metadata.as_ref().and_then(|m| m.get("year")),
        Some(&Value::Int(2025)),
    );
}

#[test]
fn closed_world_filter_distinguishes_neq_from_not_eq() {
    let record = meta(&[("year", Value::Int(2026))]); // no `author` field

    let neq = Filter::neq("author", Value::String("ada".to_string()));
    let not_eq = Filter::not(Filter::eq("author", Value::String("ada".to_string())));

    assert!(!eval_filter(&neq, Some(&record))); // absent field => false
    assert!(eval_filter(&not_eq, Some(&record))); // missing-or-non-matching => true
}

#[test]
fn nested_and_or_not_and_range_filters_evaluate() {
    let record = meta(&[
        ("published", Value::Bool(true)),
        ("year", Value::Int(2026)),
        ("lang", Value::String("rust".to_string())),
    ]);

    let all = Filter::and(vec![
        Filter::eq("published", Value::Bool(true)),
        Filter::gte("year", Value::Int(2020)),
        Filter::is_in(
            "lang",
            vec![
                Value::String("rust".to_string()),
                Value::String("zig".to_string()),
            ],
        ),
        Filter::not(Filter::gt("year", Value::Int(2030))),
    ]);
    assert!(eval_filter(&all, Some(&record)));

    let none = Filter::or(vec![
        Filter::eq("published", Value::Bool(false)),
        Filter::lt("year", Value::Int(2000)),
    ]);
    assert!(!eval_filter(&none, Some(&record)));
}

#[test]
fn empty_and_is_vacuously_true_empty_or_is_false() {
    // The documented `Filter` contract: an empty `And` matches everything
    // (vacuous truth), an empty `Or` matches nothing. A consumer-side evaluator
    // built from the public surface must be able to honour both.
    let record = meta(&[("x", Value::Int(1))]);
    assert!(eval_filter(&Filter::and(vec![]), Some(&record)));
    assert!(!eval_filter(&Filter::or(vec![]), Some(&record)));
    // And on a record with no metadata at all.
    assert!(eval_filter(&Filter::and(vec![]), None));
    assert!(!eval_filter(&Filter::or(vec![]), None));
}

#[cfg(feature = "serde")]
#[test]
fn snapshot_round_trips_like_a_persistence_layer() {
    // Stand-in for `iqdb-persist`: serialize a record aggregate to JSON and back.
    let record: (VectorId, Vector, Metadata, SearchParams) = (
        VectorId::try_from(vec![0xde, 0xad]).unwrap(),
        Vector::new(vec![0.1, 0.2, 0.3]).unwrap(),
        meta(&[("k", Value::Int(1))]),
        SearchParams {
            filter: Some(Filter::gte("year", Value::Int(2020))),
            ..SearchParams::new(5, DistanceMetric::Cosine)
        },
    );
    let json = serde_json::to_string(&record).unwrap();
    let back: (VectorId, Vector, Metadata, SearchParams) = serde_json::from_str(&json).unwrap();
    assert_eq!(record, back);
}

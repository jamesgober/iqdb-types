//! Edge-case coverage — the corners the happy-path and property tests don't
//! pin explicitly. Each test documents the contract it locks.

#![allow(clippy::unwrap_used)]

use iqdb_types::{
    DistanceMetric, Filter, Hit, IqdbError, Metadata, SearchParams, Value, Vector, VectorId,
    VectorRef,
};

// ---- Value -------------------------------------------------------------

#[test]
fn value_float_nan_breaks_reflexive_equality_by_design() {
    // `Value` holds an `f64`, so it is `PartialEq` but not `Eq`. A `NaN` is not
    // equal to itself — downstream filters must treat NaN comparisons as
    // non-matching (the closed-world rule).
    let nan = Value::Float(f64::NAN);
    assert_ne!(nan, Value::Float(f64::NAN));
}

#[test]
fn value_variants_of_different_kinds_are_never_equal() {
    assert_ne!(Value::Int(1), Value::Float(1.0));
    assert_ne!(Value::Bool(true), Value::Int(1));
    assert_ne!(Value::Null, Value::Bool(false));
    assert_ne!(Value::String("1".to_string()), Value::Int(1));
}

// ---- Metadata ----------------------------------------------------------

#[test]
fn metadata_collect_keeps_the_last_value_for_a_duplicate_key() {
    // FromIterator collects into a BTreeMap: a later pair overwrites an earlier.
    let meta: Metadata = [
        ("k".to_string(), Value::Int(1)),
        ("k".to_string(), Value::Int(2)),
    ]
    .into_iter()
    .collect();
    assert_eq!(meta.len(), 1);
    assert_eq!(meta.get("k"), Some(&Value::Int(2)));
}

#[test]
fn metadata_default_is_empty_and_queryable() {
    let m = Metadata::default();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
    assert_eq!(m.get("anything"), None);
    assert_eq!(m.iter().count(), 0);
}

// ---- Vector ------------------------------------------------------------

#[test]
fn vector_handles_single_and_large_dimensions() {
    let one = Vector::new(vec![0.0]).unwrap();
    assert_eq!(one.dim(), 1);
    assert!(!one.is_empty());

    let big: Vec<f32> = (0..4096).map(|i| i as f32).collect();
    let v = Vector::new(big.clone()).unwrap();
    assert_eq!(v.dim(), 4096);
    assert_eq!(v.into_inner(), big); // round-trips the exact buffer
}

#[test]
fn vector_accepts_finite_extremes_and_subnormals() {
    assert!(Vector::new(vec![f32::MIN, -0.0, 0.0, f32::MAX]).is_ok());
    assert!(Vector::new(vec![f32::MIN_POSITIVE / 2.0]).is_ok()); // subnormal
}

#[test]
fn vector_ref_permits_an_empty_view() {
    // Unlike `Vector` (which rejects empty at construction), `VectorRef` is a
    // non-validating transient view — an empty borrow is legal.
    let empty: [f32; 0] = [];
    let r = VectorRef::from(&empty[..]);
    assert!(r.is_empty());
    assert_eq!(r.dim(), 0);
}

// ---- VectorId ----------------------------------------------------------

#[test]
fn vector_id_single_byte_and_max_u64_render() {
    assert_eq!(VectorId::try_from(vec![0xff]).unwrap().to_string(), "ff");
    assert_eq!(VectorId::from(u64::MAX).to_string(), u64::MAX.to_string());
}

// ---- Filter ------------------------------------------------------------

#[test]
fn empty_filter_combinators_construct_the_expected_shape() {
    // Documented semantics (evaluated by the engine): empty `And` is vacuously
    // true, empty `Or` is false. Here we pin that the constructors accept an
    // empty Vec and produce the canonical variant.
    assert_eq!(Filter::and(vec![]), Filter::And(vec![]));
    assert_eq!(Filter::or(vec![]), Filter::Or(vec![]));
}

// ---- SearchParams / Hit ------------------------------------------------

#[test]
fn search_params_tier1_leaves_optionals_unset() {
    let p = SearchParams::new(0, DistanceMetric::Hamming);
    assert_eq!(p.k, 0); // a degenerate but legal request
    assert_eq!(p.ef, None);
    assert!(p.filter.is_none());
}

#[test]
fn hit_distance_may_be_negative_or_zero() {
    // Under `DotProduct` the engine reports a negative-dot "distance"; `Hit`
    // stores whatever the metric produced.
    assert_eq!(Hit::new(VectorId::from(1u64), -3.5).distance, -3.5);
    assert_eq!(Hit::new(VectorId::from(2u64), 0.0).distance, 0.0);
}

// ---- IqdbError ---------------------------------------------------------

#[test]
fn error_is_copy_and_equatable_with_struct_fields() {
    let e = IqdbError::ResourceLimitExceeded {
        kind: "total_vectors",
        max: 10,
        found: 11,
    };
    let copy = e; // `IqdbError: Copy`
    assert_eq!(e, copy);
    assert_ne!(IqdbError::NotFound, IqdbError::Duplicate);
}

// ---- serde -------------------------------------------------------------

#[cfg(feature = "serde")]
#[test]
fn vector_ref_serializes_as_its_slice() {
    // `VectorRef` is `Serialize`-only and is a newtype over `&[f32]`, so it
    // serializes transparently as a JSON array.
    let data = [1.0_f32, 2.0, 3.0];
    let json = serde_json::to_string(&VectorRef::from(&data[..])).unwrap();
    assert_eq!(json, "[1.0,2.0,3.0]");
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_null_and_empty_metadata() {
    let null = serde_json::to_string(&Value::Null).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&null).unwrap(), Value::Null);

    let empty = Metadata::default();
    let json = serde_json::to_string(&empty).unwrap();
    assert_eq!(json, "{}");
    assert_eq!(serde_json::from_str::<Metadata>(&json).unwrap(), empty);
}

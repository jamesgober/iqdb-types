//! End-to-end smoke coverage for the `iqdb-types` public surface.
//!
//! This is an integration test: it sees only the crate's public API plus the
//! `[dev-dependencies]`, so it doubles as a check that every type a downstream
//! crate needs is actually re-exported. It constructs each core type, exercises
//! the `VectorId` conversions (including the empty-key error path), builds a
//! nested `Filter` tree, round-trips `Metadata`, and asserts the `IqdbError`
//! `Display`/`ForgeError` contract. The `serde` block runs only under
//! `--features serde`.

use error_forge::ForgeError;
use iqdb_types::{
    DistanceMetric, Filter, Hit, IqdbError, Metadata, SearchParams, VERSION, Value, Vector,
    VectorId, VectorRef,
};

#[test]
fn constructs_each_core_type() {
    // Arrange / Act
    let owned = Vector::new(vec![0.1, 0.2, 0.3]).unwrap();
    let data = [0.1, 0.2, 0.3];
    let borrowed = VectorRef::from(&data[..]);
    let metric = DistanceMetric::Cosine;
    let params = SearchParams::new(5, metric);
    let hit = Hit::new(VectorId::from(1u64), 0.25);

    // Assert
    assert_eq!(owned.dim(), 3);
    assert_eq!(borrowed.dim(), 3);
    assert_eq!(owned.as_slice(), borrowed.as_slice());
    assert_eq!(params.k, 5);
    assert_eq!(params.metric, DistanceMetric::Cosine);
    assert_eq!(params.ef, None);
    assert!(params.filter.is_none());
    assert_eq!(hit.id, VectorId::U64(1));
    assert_eq!(hit.distance, 0.25);
    assert!(hit.metadata.is_none());
}

#[test]
fn vector_new_rejects_empty_and_non_finite() {
    // Empty is rejected.
    assert_eq!(
        Vector::new(Vec::new()).unwrap_err(),
        IqdbError::InvalidVector,
    );
    // NaN is rejected.
    assert_eq!(
        Vector::new(vec![1.0, f32::NAN, 0.0]).unwrap_err(),
        IqdbError::InvalidVector,
    );
    // +Inf is rejected.
    assert_eq!(
        Vector::new(vec![f32::INFINITY]).unwrap_err(),
        IqdbError::InvalidVector,
    );
    // -Inf is rejected.
    assert_eq!(
        Vector::new(vec![f32::NEG_INFINITY]).unwrap_err(),
        IqdbError::InvalidVector,
    );
    // Single non-finite component anywhere in the buffer is rejected.
    assert_eq!(
        Vector::new(vec![0.0, 0.0, f32::NAN, 0.0]).unwrap_err(),
        IqdbError::InvalidVector,
    );
    // Finite values, including signed zero and very small / very large
    // finite values, are accepted.
    assert!(Vector::new(vec![0.0_f32, -0.0]).is_ok());
    assert!(Vector::new(vec![f32::MIN, f32::MAX]).is_ok());
    assert!(Vector::new(vec![1.0]).is_ok());
}

#[test]
fn vector_try_from_matches_new() {
    // The TryFrom impl is the ergonomic alias for Vector::new; both must
    // accept and reject the same inputs.
    let ok: Vector = vec![1.0_f32, 2.0].try_into().unwrap();
    assert_eq!(ok.dim(), 2);

    let err: Result<Vector, IqdbError> = vec![f32::NAN].try_into();
    assert_eq!(err.unwrap_err(), IqdbError::InvalidVector);

    let empty: Result<Vector, IqdbError> = Vec::<f32>::new().try_into();
    assert_eq!(empty.unwrap_err(), IqdbError::InvalidVector);
}

#[test]
fn vector_id_conversions_round_trip() {
    // From<u64>
    assert_eq!(VectorId::from(7u64), VectorId::U64(7));

    // TryFrom<Vec<u8>> on a non-empty key.
    let key = VectorId::try_from(vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(
        key,
        Ok(VectorId::Bytes(
            vec![0xde, 0xad, 0xbe, 0xef].into_boxed_slice()
        ))
    );

    // The empty-key error path. Empty `VectorId::Bytes` is a malformed
    // identifier, not a malformed vector — it surfaces as
    // `InvalidConfig { reason }`, not `InvalidVector`.
    let err = VectorId::try_from(Vec::new()).unwrap_err();
    assert!(
        matches!(err, IqdbError::InvalidConfig { .. }),
        "expected InvalidConfig, got {err:?}",
    );
}

#[test]
fn builds_a_nested_filter_tree() {
    // author == "ada" AND NOT (year > 2000)
    let filter = Filter::and(vec![
        Filter::eq("author", Value::String("ada".to_string())),
        Filter::not(Filter::gt("year", Value::Int(2000))),
    ]);

    assert_eq!(
        filter,
        Filter::And(vec![
            Filter::Eq {
                field: "author".to_string(),
                value: Value::String("ada".to_string())
            },
            Filter::Not(Box::new(Filter::Gt {
                field: "year".to_string(),
                value: Value::Int(2000),
            })),
        ]),
    );
}

#[test]
fn metadata_round_trips_through_an_iterator() {
    let meta: Metadata = [
        ("title".to_string(), Value::String("intro".to_string())),
        ("year".to_string(), Value::Int(2026)),
        ("published".to_string(), Value::Bool(true)),
    ]
    .into_iter()
    .collect();

    assert_eq!(meta.len(), 3);
    assert!(!meta.is_empty());
    assert_eq!(meta.get("year"), Some(&Value::Int(2026)));
    assert_eq!(meta.get("missing"), None);

    // BTreeMap orders keys, so iteration is deterministic.
    let keys: Vec<&String> = meta.iter().map(|(key, _)| key).collect();
    assert_eq!(keys, vec!["published", "title", "year"]);
}

#[test]
fn error_reports_its_message_and_kind() {
    let err = IqdbError::DimensionMismatch {
        expected: 3,
        found: 2,
    };

    assert_eq!(
        err.to_string(),
        "vector dimension mismatch: expected 3, found 2"
    );
    assert_eq!(err.kind(), "DimensionMismatch");
}

#[test]
fn each_error_variant_carries_a_distinct_caption() {
    // Every variant gets a human-readable caption distinct from every other,
    // so the central ForgeError hook can describe what went wrong without
    // re-encoding the variant. A single shared caption would defeat the trait.
    let variants = [
        IqdbError::DimensionMismatch {
            expected: 1,
            found: 2,
        },
        IqdbError::InvalidVector,
        IqdbError::InvalidConfig {
            reason: "smoke-test reason",
        },
        IqdbError::NotFound,
        IqdbError::Duplicate,
        IqdbError::InvalidMetric,
        IqdbError::InvalidFilter,
        IqdbError::ResourceLimitExceeded {
            kind: "id_bytes",
            max: 1,
            found: 2,
        },
    ];

    let captions: Vec<&'static str> = variants.iter().map(ForgeError::caption).collect();

    for (i, a) in captions.iter().enumerate() {
        assert!(!a.is_empty(), "variant {i} has an empty caption");
        for (j, b) in captions.iter().enumerate().skip(i + 1) {
            assert_ne!(
                a, b,
                "variants {i} and {j} share caption {a:?} — captions must be distinct",
            );
        }
    }

    // Spot-check one variant's wording is meaningful (not just the type name).
    assert_eq!(
        IqdbError::DimensionMismatch {
            expected: 0,
            found: 0,
        }
        .caption(),
        "vector dimension does not match the index",
    );
}

#[test]
fn version_is_present() {
    assert!(!VERSION.is_empty());
    assert_eq!(VERSION.split('.').count(), 3);
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_id_and_filter() {
    let id = VectorId::Bytes(vec![1, 2, 3].into_boxed_slice());
    let id_json = serde_json::to_string(&id).expect("serialize VectorId");
    let id_back: VectorId = serde_json::from_str(&id_json).expect("deserialize VectorId");
    assert_eq!(id, id_back);

    let filter = Filter::or(vec![
        Filter::is_in("year", vec![Value::Int(2025), Value::Int(2026)]),
        Filter::neq("status", Value::String("draft".to_string())),
    ]);
    let filter_json = serde_json::to_string(&filter).expect("serialize Filter");
    let filter_back: Filter = serde_json::from_str(&filter_json).expect("deserialize Filter");
    assert_eq!(filter, filter_back);
}

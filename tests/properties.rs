//! Property-based coverage of the `iqdb-types` invariants.
//!
//! `dev/DIRECTIVES.md` §5 and §8 require every core type contract to be
//! property-tested, not only covered by examples: a breaking change here
//! cascades through the whole iQDB family, so the guarantees this crate makes —
//! validation, ordering, `Display` shape, and `serde` round-trips — are checked
//! against generated inputs rather than a handful of hand-picked cases.
//!
//! Integration-test crate: it sees only the public surface plus
//! `[dev-dependencies]`.

#![allow(clippy::unwrap_used)]

use iqdb_types::{DistanceMetric, IqdbError, Metadata, Value, Vector, VectorId, VectorRef};
use proptest::prelude::*;

/// A single finite `f32`. Excludes NaN and ±∞ by construction (a range
/// strategy only yields values inside its bounds).
fn finite_f32() -> impl Strategy<Value = f32> {
    -1.0e30f32..1.0e30f32
}

/// A non-empty buffer of finite `f32` components — the shape `Vector::new`
/// must always accept.
fn finite_vec() -> impl Strategy<Value = Vec<f32>> {
    prop::collection::vec(finite_f32(), 1..64)
}

proptest! {
    // ---- Vector ---------------------------------------------------------

    /// `Vector::new` accepts every non-empty, all-finite buffer and preserves
    /// its contents exactly through every accessor.
    #[test]
    fn vector_new_accepts_finite_and_round_trips(data in finite_vec()) {
        let v = Vector::new(data.clone()).unwrap();
        prop_assert_eq!(v.len(), data.len());
        prop_assert_eq!(v.dim(), data.len());
        prop_assert!(!v.is_empty());
        prop_assert_eq!(v.as_slice(), data.as_slice());
        prop_assert_eq!(v.into_inner(), data);
    }

    /// A non-finite component anywhere in the buffer is rejected with
    /// `InvalidVector`, regardless of position.
    #[test]
    fn vector_new_rejects_any_non_finite(
        mut data in finite_vec(),
        idx in any::<prop::sample::Index>(),
        bad in prop_oneof![Just(f32::NAN), Just(f32::INFINITY), Just(f32::NEG_INFINITY)],
    ) {
        let i = idx.index(data.len());
        data[i] = bad;
        prop_assert_eq!(Vector::new(data).unwrap_err(), IqdbError::InvalidVector);
    }

    /// `TryFrom<Vec<f32>>` is exactly `Vector::new`: it accepts and rejects the
    /// same inputs.
    #[test]
    fn vector_try_from_agrees_with_new(data in finite_vec()) {
        let via_new = Vector::new(data.clone());
        let via_try: Result<Vector, IqdbError> = data.try_into();
        prop_assert_eq!(via_new.unwrap(), via_try.unwrap());
    }

    /// `VectorRef` is a faithful zero-copy view: every accessor reports the
    /// underlying slice unchanged.
    #[test]
    fn vector_ref_views_slice_faithfully(data in prop::collection::vec(finite_f32(), 0..64)) {
        let r = VectorRef::from(data.as_slice());
        prop_assert_eq!(r.len(), data.len());
        prop_assert_eq!(r.dim(), data.len());
        prop_assert_eq!(r.is_empty(), data.is_empty());
        prop_assert_eq!(r.as_slice(), data.as_slice());
        prop_assert_eq!(r.into_inner(), data.as_slice());
    }

    // ---- VectorId -------------------------------------------------------

    /// A `U64` id renders as its decimal integer and compares equal to the
    /// canonical variant.
    #[test]
    fn vector_id_u64_display_is_decimal(n in any::<u64>()) {
        let id = VectorId::from(n);
        prop_assert_eq!(id.clone(), VectorId::U64(n));
        prop_assert_eq!(id.to_string(), n.to_string());
    }

    /// A non-empty byte key builds a `Bytes` id that renders as fixed-width
    /// lowercase hex (two chars per byte, no prefix, no separators).
    #[test]
    fn vector_id_bytes_display_is_lower_hex(bytes in prop::collection::vec(any::<u8>(), 1..64)) {
        let id = VectorId::try_from(bytes.clone()).unwrap();
        prop_assert_eq!(id.clone(), VectorId::Bytes(bytes.clone().into_boxed_slice()));

        let rendered = id.to_string();
        prop_assert_eq!(rendered.len(), bytes.len() * 2);
        prop_assert!(
            rendered.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "rendering must be lowercase hex: {rendered}",
        );
        // The hex decodes back to the original bytes.
        let decoded: Vec<u8> = (0..rendered.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&rendered[i..i + 2], 16).unwrap())
            .collect();
        prop_assert_eq!(decoded, bytes);
    }

    /// `Debug` and `Display` never collapse to the same shape for `Bytes` —
    /// logs written through `Display` must not regress to raw byte arrays.
    #[test]
    fn vector_id_bytes_debug_differs_from_display(bytes in prop::collection::vec(any::<u8>(), 1..32)) {
        let id = VectorId::try_from(bytes).unwrap();
        prop_assert_ne!(format!("{id}"), format!("{id:?}"));
    }

    // ---- Metadata -------------------------------------------------------

    /// `Metadata` iterates in ascending key order and returns every inserted
    /// value by key; a key outside the generated alphabet is always absent.
    #[test]
    fn metadata_is_key_ordered_and_queryable(
        map in prop::collection::btree_map("[a-z]{1,8}", any::<i64>(), 0..32)
    ) {
        let meta: Metadata = map.iter().map(|(k, v)| (k.clone(), Value::Int(*v))).collect();

        prop_assert_eq!(meta.len(), map.len());
        prop_assert_eq!(meta.is_empty(), map.is_empty());

        let keys: Vec<String> = meta.iter().map(|(k, _)| k.clone()).collect();
        prop_assert!(keys.windows(2).all(|w| w[0] < w[1]), "keys must be strictly ascending");

        for (k, v) in &map {
            // Bind the expected value to a local: `prop_assert_eq!` moves its
            // operands into bindings, so `Some(&Value::Int(*v))` as a temporary
            // would be dropped while borrowed (E0716) on the 1.87 MSRV.
            let expected = Value::Int(*v);
            prop_assert_eq!(meta.get(k), Some(&expected));
        }
        // "123" cannot match the [a-z] key alphabet, so it is always absent.
        prop_assert_eq!(meta.get("123"), None);
    }

    // ---- DistanceMetric -------------------------------------------------

    /// `DistanceMetric` honours the `Eq`/`Hash` contract: equal values hash
    /// equal, and a value equals its own clone.
    #[test]
    fn distance_metric_eq_hash_consistent(a in metric(), b in metric()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        prop_assert_eq!(a, a);
        if a == b {
            let mut ha = DefaultHasher::new();
            let mut hb = DefaultHasher::new();
            a.hash(&mut ha);
            b.hash(&mut hb);
            prop_assert_eq!(ha.finish(), hb.finish());
        }
    }
}

/// Strategy over the five `DistanceMetric` variants.
fn metric() -> impl Strategy<Value = DistanceMetric> {
    prop_oneof![
        Just(DistanceMetric::Cosine),
        Just(DistanceMetric::DotProduct),
        Just(DistanceMetric::Euclidean),
        Just(DistanceMetric::Manhattan),
        Just(DistanceMetric::Hamming),
    ]
}

// ---- serde round-trips --------------------------------------------------
//
// dev/DIRECTIVES.md §8 requires a serde round-trip property for every public
// type. These run only under `--features serde`.

#[cfg(feature = "serde")]
mod serde_props {
    use super::*;
    use iqdb_types::{Filter, Hit, SearchParams};
    use proptest::test_runner::TestCaseError;

    /// JSON round-trip: `from_str(to_string(v)) == v`.
    fn round_trip<T>(value: &T) -> Result<(), TestCaseError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(value, &back);
        Ok(())
    }

    fn key() -> impl Strategy<Value = String> {
        "[a-z]{1,8}".prop_map(|s| s)
    }

    /// A scalar `Value`. `Float` is drawn from a finite range (no NaN/∞, which
    /// JSON cannot represent and which would also break `PartialEq`) and then
    /// normalized through one JSON round-trip: serde_json's own float precision
    /// can shift an extreme-magnitude `f64` by a ULP, which is out of scope for
    /// *this* test — we are asserting that the `Value` derive preserves whatever
    /// serde_json can represent, not re-testing serde_json's float formatting.
    fn value() -> impl Strategy<Value = Value> {
        let json_stable_f64 = (-1.0e30f64..1.0e30f64).prop_map(|f| {
            let s = serde_json::to_string(&f).unwrap();
            serde_json::from_str::<f64>(&s).unwrap()
        });
        prop_oneof![
            "[ -~]{0,16}".prop_map(Value::String),
            any::<i64>().prop_map(Value::Int),
            json_stable_f64.prop_map(Value::Float),
            any::<bool>().prop_map(Value::Bool),
            Just(Value::Null),
        ]
    }

    fn vector() -> impl Strategy<Value = Vector> {
        finite_vec().prop_map(|d| Vector::new(d).unwrap())
    }

    fn vector_id() -> impl Strategy<Value = VectorId> {
        prop_oneof![
            any::<u64>().prop_map(VectorId::from),
            prop::collection::vec(any::<u8>(), 1..16).prop_map(|b| VectorId::try_from(b).unwrap()),
        ]
    }

    fn metadata() -> impl Strategy<Value = Metadata> {
        prop::collection::btree_map(key(), value(), 0..8).prop_map(|m| m.into_iter().collect())
    }

    /// A bounded-depth `Filter` tree: leaves plus `And`/`Or`/`Not` combinators.
    fn filter() -> impl Strategy<Value = Filter> {
        let leaf = prop_oneof![
            (key(), value()).prop_map(|(k, v)| Filter::eq(k, v)),
            (key(), value()).prop_map(|(k, v)| Filter::neq(k, v)),
            (key(), value()).prop_map(|(k, v)| Filter::lt(k, v)),
            (key(), value()).prop_map(|(k, v)| Filter::lte(k, v)),
            (key(), value()).prop_map(|(k, v)| Filter::gt(k, v)),
            (key(), value()).prop_map(|(k, v)| Filter::gte(k, v)),
            (key(), prop::collection::vec(value(), 0..4)).prop_map(|(k, vs)| Filter::is_in(k, vs)),
        ];
        leaf.prop_recursive(4, 32, 4, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..4).prop_map(Filter::and),
                prop::collection::vec(inner.clone(), 0..4).prop_map(Filter::or),
                inner.prop_map(Filter::not),
            ]
        })
    }

    fn hit() -> impl Strategy<Value = Hit> {
        (vector_id(), finite_f32(), prop::option::of(metadata())).prop_map(
            |(id, distance, meta)| Hit {
                id,
                distance,
                metadata: meta,
            },
        )
    }

    fn search_params() -> impl Strategy<Value = SearchParams> {
        (
            any::<usize>(),
            prop::option::of(any::<usize>()),
            metric(),
            prop::option::of(filter()),
        )
            .prop_map(|(k, ef, metric, filter)| SearchParams {
                k,
                ef,
                metric,
                filter,
            })
    }

    proptest! {
        #[test]
        fn value_round_trips(v in value()) { round_trip(&v)? }

        #[test]
        fn vector_round_trips(v in vector()) { round_trip(&v)? }

        #[test]
        fn vector_id_round_trips(id in vector_id()) { round_trip(&id)? }

        #[test]
        fn distance_metric_round_trips(m in metric()) { round_trip(&m)? }

        #[test]
        fn metadata_round_trips(m in metadata()) { round_trip(&m)? }

        #[test]
        fn filter_round_trips(f in filter()) { round_trip(&f)? }

        #[test]
        fn hit_round_trips(h in hit()) { round_trip(&h)? }

        #[test]
        fn search_params_round_trips(p in search_params()) { round_trip(&p)? }
    }
}

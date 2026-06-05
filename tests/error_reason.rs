//! `IqdbError::InvalidConfig` carries a reason (audit finding L5).
//!
//! Pre-L5 the variant was opaque: every config-shaped failure surfaced as
//! the same `IqdbError::InvalidConfig` with the same `Display` string,
//! forcing callers to grep stack traces to disambiguate "dim must be set"
//! from "quantizer not trained" from "output buffer length mismatch".
//! Post-L5 the variant carries a `&'static str` reason, which is included
//! in `Display`. `&'static str` keeps the error `Copy`.

#![allow(clippy::unwrap_used)]

use iqdb_types::IqdbError;

#[test]
fn invalid_config_carries_a_static_reason() {
    let err = IqdbError::InvalidConfig {
        reason: "dim must be greater than zero",
    };
    // The variant is a struct variant — match on it to prove the field exists.
    let reason = match err {
        IqdbError::InvalidConfig { reason } => reason,
        other => panic!("expected InvalidConfig, got {other:?}"),
    };
    assert_eq!(reason, "dim must be greater than zero");
}

#[test]
fn display_includes_the_reason() {
    let err = IqdbError::InvalidConfig {
        reason: "dim must be set before building",
    };
    let rendered = err.to_string();
    // The "invalid configuration" prefix is preserved for operators who
    // already grep for it; the reason is appended after a separator.
    assert!(
        rendered.contains("invalid configuration"),
        "missing variant prefix: {rendered}",
    );
    assert!(
        rendered.contains("dim must be set before building"),
        "missing reason: {rendered}",
    );
}

#[test]
fn distinct_reasons_render_distinctly() {
    let a = IqdbError::InvalidConfig {
        reason: "dim must be greater than zero",
    };
    let b = IqdbError::InvalidConfig {
        reason: "output buffer length mismatch",
    };
    assert_ne!(a.to_string(), b.to_string());
}

#[test]
fn error_remains_copy() {
    // `IqdbError: Copy` is part of the cross-crate contract — every Result
    // shorthand and every test that asserts `assert_eq!(err, IqdbError::...)`
    // depends on it. The `InvalidConfig::reason` field MUST be
    // `&'static str` (not `String` or `Cow<'_, str>`) so the variant stays
    // `Copy`. The same constraint applies to the
    // `ResourceLimitExceeded::kind` field — a `String` there would also
    // break `Copy` for the whole enum. Asserting the bound at the enum
    // level catches a regression on either variant.
    fn assert_copy<T: Copy>() {}
    assert_copy::<IqdbError>();

    let err = IqdbError::InvalidConfig { reason: "x" };
    let copied = err;
    let _ = err; // still usable after copy
    let _ = copied;

    // Parallel sanity check for `ResourceLimitExceeded` so the contract
    // is explicit at the call site, not only implied by the enum-level
    // `assert_copy::<IqdbError>()`.
    let limit_err = IqdbError::ResourceLimitExceeded {
        kind: "id_bytes",
        max: 1,
        found: 2,
    };
    let limit_copy = limit_err;
    let _ = limit_err;
    let _ = limit_copy;
}

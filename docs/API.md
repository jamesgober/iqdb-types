# iqdb-types &mdash; API Reference

> Complete reference for the public surface of `iqdb-types` as of **v0.2.0**.
> **Status: pre-1.0.** The surface is refined across the 0.x series and frozen at `1.0.0`.

## Table of Contents

- [Overview](#overview)
- [Vectors](#vectors)
- [Identifiers](#identifiers)
- [Metadata](#metadata)
- [Distance metrics](#distance-metrics)
- [Filters](#filters)
- [Search parameters](#search-parameters)
- [Hits](#hits)
- [Errors](#errors)
- [Feature flags](#feature-flags)

---

## Overview

`iqdb-types` defines the vocabulary the entire iQDB vector-database family speaks. When you read the docs for `iqdb-hnsw` or `iqdb-flat`, every type you meet — `Vector`, `VectorId`, `DistanceMetric`, `Hit`, `Metadata`, `SearchParams` — is defined here. The crate is pure data: no engine, no storage, no I/O. Its only runtime dependency is `error-forge`.

```rust
use iqdb_types::{DistanceMetric, Filter, SearchParams, Value, Vector, VectorId};

let embedding = Vector::new(vec![0.1, 0.2, 0.3]).unwrap();
let id = VectorId::from(1u64);
let params = SearchParams {
    filter: Some(Filter::eq("published", Value::Bool(true))),
    ..SearchParams::new(3, DistanceMetric::Cosine)
};
```

`iqdb_types::VERSION` exposes the crate's compile-time `CARGO_PKG_VERSION` for diagnostics and version-skew checks.

---

## Vectors

Dense `f32` vectors, owned (`Vector`) and borrowed (`VectorRef<'a>`).

```rust
pub struct Vector(/* private */);

impl Vector {
    pub fn new(data: Vec<f32>) -> Result<Self>; // rejects empty / non-finite
    pub fn new_unchecked(data: Vec<f32>) -> Self; // `testing` feature only
    pub fn as_slice(&self) -> &[f32];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn dim(&self) -> usize;
    pub fn into_inner(self) -> Vec<f32>;
}
impl TryFrom<Vec<f32>> for Vector; // Error = IqdbError

pub struct VectorRef<'a>(/* private */);

impl<'a> VectorRef<'a> {
    pub fn as_slice(&self) -> &[f32];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn dim(&self) -> usize;
    pub fn into_inner(self) -> &'a [f32];
}
impl<'a> From<&'a [f32]> for VectorRef<'a>;
```

**Validation contract.** `Vector::new` (and `TryFrom`) reject an empty vector or any non-finite component (NaN, ±∞) with [`IqdbError::InvalidVector`](#errors). Validation happens at the type boundary so downstream math never has to defend against invalid floats. `new_unchecked` bypasses this and is reachable only under the `testing` feature.

---

## Identifiers

```rust
pub enum VectorId {
    U64(u64),
    Bytes(Box<[u8]>),
}
impl From<u64> for VectorId;
impl TryFrom<Vec<u8>> for VectorId; // rejects an empty byte key
impl fmt::Display for VectorId;
```

A stable id: either a 64-bit integer or an opaque, non-empty byte key.

---

## Metadata

```rust
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

pub struct Metadata(/* private, ordered */);

impl Metadata {
    pub fn get(&self, key: &str) -> Option<&Value>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)>;
}
impl From<BTreeMap<String, Value>> for Metadata;
impl FromIterator<(String, Value)> for Metadata;
```

`Metadata` is an immutable, ordered (`BTreeMap`-backed) map of scalar attributes. Construct a new value; there are no in-place mutators.

---

## Distance metrics

```rust
pub enum DistanceMetric {
    Cosine,
    DotProduct,
    Euclidean,
    Manhattan,
    Hamming,
}
```

A metric tag only — the actual kernels live in `iqdb-distance`.

---

## Filters

```rust
pub enum Filter { /* leaves + And/Or/Not */ }

impl Filter {
    pub fn eq(field: impl Into<String>, value: Value) -> Self;
    pub fn neq(field: impl Into<String>, value: Value) -> Self;
    pub fn lt(field: impl Into<String>, value: Value) -> Self;
    pub fn lte(field: impl Into<String>, value: Value) -> Self;
    pub fn gt(field: impl Into<String>, value: Value) -> Self;
    pub fn gte(field: impl Into<String>, value: Value) -> Self;
    pub fn is_in(field: impl Into<String>, values: Vec<Value>) -> Self;
    pub fn and(filters: Vec<Filter>) -> Self;
    pub fn or(filters: Vec<Filter>) -> Self;
    pub fn not(inner: Filter) -> Self;
}
```

A boolean expression tree over metadata. **Closed-world semantics:** every leaf comparison on an absent field evaluates to `false`. So `Filter::neq("author", "ada")` does **not** match records lacking an `author` field — use `Filter::not(Filter::eq("author", "ada"))` for that.

---

## Search parameters

```rust
pub struct SearchParams {
    pub k: usize,
    pub ef: Option<usize>,        // search-breadth knob (e.g. HNSW efSearch)
    pub metric: DistanceMetric,
    pub filter: Option<Filter>,
}
impl SearchParams {
    pub fn new(k: usize, metric: DistanceMetric) -> Self;
}
```

---

## Hits

```rust
pub struct Hit {
    pub id: VectorId,
    pub distance: f32,
    pub metadata: Option<Metadata>,
}
impl Hit {
    pub fn new(id: VectorId, distance: f32) -> Self;
}
```

One search result: an id, a distance, and optional metadata.

---

## Errors

```rust
pub enum IqdbError {
    DimensionMismatch { /* .. */ },
    InvalidVector,
    InvalidConfig { /* .. */ },
    NotFound,
    Duplicate,
    InvalidMetric,
    InvalidFilter,
    ResourceLimitExceeded { /* .. */ },
}
pub type Result<T> = core::result::Result<T, IqdbError>;
```

`IqdbError` implements `error_forge::ForgeError` (`kind()`, `caption()`), `std::error::Error`, and `Display`. Each variant has a distinct human-readable `caption()`; use `kind()` for a stable machine identifier.

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `serde` | no | Derives `Serialize`/`Deserialize` on the public types. `VectorRef` is `Serialize` only — a borrowed view has nowhere to own decoded data. |
| `testing` | no | Exposes `Vector::new_unchecked`, the test-only escape hatch for building a `Vector` without validation. Production builds cannot reach it. |

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>

# iqdb-types &mdash; API Reference

> Complete reference for **every** public item in `iqdb-types` as of **v1.0.0**:
> what it is, its parameters and return shape, the traits it implements, and
> worked examples for each use case.
>
> **Status: stable (1.0).** The public API is committed under SemVer for the 1.x
> series — no breaking changes until 2.0 (the frozen surface is recorded in
> `dev/ROADMAP.md`). Only additive, non-breaking changes are made within 1.x.
> `DistanceMetric` and `IqdbError` are `#[non_exhaustive]`.

## Table of Contents

- [Overview](#overview)
- [Crate constants](#crate-constants)
  - [`VERSION`](#version)
- [Vectors](#vectors)
  - [`Vector`](#vector)
  - [`VectorRef`](#vectorref)
- [Identifiers](#identifiers)
  - [`VectorId`](#vectorid)
- [Metadata](#metadata)
  - [`Value`](#value)
  - [`Metadata`](#metadata-1)
- [Distance](#distance)
  - [`DistanceMetric`](#distancemetric)
- [Filters](#filters)
  - [`Filter`](#filter)
- [Search](#search)
  - [`SearchParams`](#searchparams)
  - [`Hit`](#hit)
- [Errors](#errors)
  - [`IqdbError`](#iqdberror)
  - [`Result`](#result)
- [Feature flags](#feature-flags)
- [Trait implementation matrix](#trait-implementation-matrix)

---

## Overview

`iqdb-types` defines the vocabulary the entire iQDB vector-database family speaks. Every type an index or the database exposes — `Vector`, `VectorId`, `DistanceMetric`, `Hit`, `Metadata`, `SearchParams`, `Filter`, `IqdbError` — is declared here. The crate is pure data: no engine, no storage, no I/O. Its only runtime dependency is `error-forge`.

```rust
use iqdb_types::{DistanceMetric, Filter, SearchParams, Value, Vector, VectorId};

// Build an embedding and a query in a few lines (the Tier-1 surface).
let embedding = Vector::new(vec![0.1, 0.2, 0.3]).unwrap();
let id = VectorId::from(1u64);
let params = SearchParams {
    filter: Some(Filter::eq("published", Value::Bool(true))),
    ..SearchParams::new(3, DistanceMetric::Cosine)
};
assert_eq!((embedding.dim(), id, params.k), (3, VectorId::U64(1), 3));
```

**Immutability.** Every type here is constructed whole and read back; none expose in-place mutators on shared state. To change a value, build a new one.

**Validation at the boundary.** `Vector` and `VectorId` validate on construction, so downstream crates never re-check the same invariants.

---

## Crate constants

### `VERSION`

```rust
pub const VERSION: &str;
```

The crate's compile-time version (`CARGO_PKG_VERSION`), a `major.minor.patch` SemVer core. Use it to report the exact `iqdb-types` build a binary links against — useful in diagnostics and version-skew checks across the family.

```rust
let v = iqdb_types::VERSION;
assert_eq!(v.split('.').count(), 3);
assert!(v.split('.').all(|part| !part.is_empty()));
```

---

## Vectors

### `Vector`

```rust
pub struct Vector(/* private: Box<[f32]> */);
```

An **owned**, validated, dense `f32` embedding — the unit of value the spine indexes. Construction validates contents once so the rest of the family can trust every `Vector` it receives. Components are stored in a `Box<[f32]>` (not a `Vec<f32>`): a `Vector` is immutable after construction, so it carries no spare capacity and is one machine word smaller than a `Vec`-backed wrapper.

**Derives / traits:** `Debug`, `Clone`, `PartialEq` (not `Eq` — it holds `f32`); `Serialize`/`Deserialize` under the `serde` feature.

#### `Vector::new`

```rust
pub fn new(data: Vec<f32>) -> Result<Vector>;
```

Builds a `Vector`, taking ownership of `data` without copying.

- **`data`** — the components. Must be non-empty and entirely finite.
- **Returns** `Ok(Vector)`, or [`Err(IqdbError::InvalidVector)`](#iqdberror) if `data` is empty **or** any component is non-finite (`NaN`, `+∞`, `−∞`).

```rust
use iqdb_types::{IqdbError, Vector};

// Valid input.
let v = Vector::new(vec![0.1, 0.2, 0.3]).unwrap();
assert_eq!(v.dim(), 3);

// Empty is rejected.
assert_eq!(Vector::new(Vec::new()).unwrap_err(), IqdbError::InvalidVector);

// Any non-finite component is rejected.
assert_eq!(Vector::new(vec![1.0, f32::NAN]).unwrap_err(), IqdbError::InvalidVector);
assert_eq!(Vector::new(vec![f32::INFINITY]).unwrap_err(), IqdbError::InvalidVector);

// Finite extremes and signed zero are fine.
assert!(Vector::new(vec![f32::MIN, 0.0, -0.0, f32::MAX]).is_ok());
```

#### `Vector::new_unchecked` *(feature `testing`)*

```rust
#[cfg(any(test, feature = "testing"))]
pub fn new_unchecked(data: Vec<f32>) -> Vector;
```

Builds a `Vector` **without** validation. Available only under the `testing` feature (a production build cannot compile a call to it). Reserved for tests that must construct otherwise-invalid vectors to exercise downstream behaviour on bad input.

- **`data`** — the components, used as-is with no checks.

```rust
# #[cfg(feature = "testing")]
# {
use iqdb_types::Vector;
let v = Vector::new_unchecked(vec![f32::NAN]); // only possible under `testing`
assert_eq!(v.len(), 1);
# }
```

#### Accessors

```rust
pub fn as_slice(&self) -> &[f32];   // borrow the components
pub fn len(&self) -> usize;          // component count
pub fn is_empty(&self) -> bool;      // always false for a `new`-built Vector
pub fn dim(&self) -> usize;          // dimensionality (== len)
pub fn into_inner(self) -> Vec<f32>; // consume, reclaim the buffer
```

```rust
use iqdb_types::Vector;

let v = Vector::new(vec![1.0, 2.0, 3.0]).unwrap();
assert_eq!(v.as_slice(), &[1.0, 2.0, 3.0]);
assert_eq!(v.len(), 3);
assert_eq!(v.dim(), 3);
assert!(!v.is_empty());
assert_eq!(v.into_inner(), vec![1.0, 2.0, 3.0]); // buffer back, no copy
```

#### `TryFrom<Vec<f32>>`

The ergonomic alias for [`Vector::new`] — identical validation, for `try_into()` call sites.

```rust
use iqdb_types::{IqdbError, Vector};

let v: Vector = vec![1.0, 0.0].try_into().unwrap();
assert_eq!(v.dim(), 2);

let bad: Result<Vector, IqdbError> = vec![f32::NAN].try_into();
assert_eq!(bad.unwrap_err(), IqdbError::InvalidVector);
```

---

### `VectorRef`

```rust
pub struct VectorRef<'a>(/* private: &'a [f32] */);
```

A **borrowed**, zero-copy view over a `&'a [f32]` — for passing a query vector through an API without taking ownership or allocating. Unlike [`Vector`], it does **not** validate (it is a transient view, not stored data).

**Derives / traits:** `Debug`, `Clone`, `Copy`, `PartialEq`; `Serialize` **only** under `serde` (a borrowed view has nowhere to own decoded data, so it cannot `Deserialize`).

#### `From<&[f32]>` and accessors

```rust
impl<'a> From<&'a [f32]> for VectorRef<'a>;

pub fn as_slice(&self) -> &[f32];
pub fn len(&self) -> usize;
pub fn is_empty(&self) -> bool;
pub fn dim(&self) -> usize;
pub fn into_inner(self) -> &'a [f32]; // returns the slice at its original lifetime
```

```rust
use iqdb_types::VectorRef;

let data = [1.0_f32, 0.0, 0.0];
let r = VectorRef::from(&data[..]);
assert_eq!(r.dim(), 3);
assert_eq!(r.as_slice(), &[1.0, 0.0, 0.0]);

// `Copy`: passing it does not move the original.
fn dims(v: VectorRef<'_>) -> usize { v.dim() }
assert_eq!(dims(r), 3);
assert_eq!(r.dim(), 3); // still usable

// The borrow keeps its original lifetime.
let slice: &[f32] = r.into_inner();
assert_eq!(slice, &data[..]);

let empty: [f32; 0] = [];
assert!(VectorRef::from(&empty[..]).is_empty());
```

---

## Identifiers

### `VectorId`

```rust
pub enum VectorId {
    U64(u64),
    Bytes(Box<[u8]>),
}
```

A stable identifier for a stored vector: either a compact 64-bit integer (a row id, a counter) or an opaque, **non-empty** byte key (a content hash, a UUID's raw bytes). The crate imposes no id scheme — use whichever your data already has.

**Derives / traits:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`; `Display`; `Serialize`/`Deserialize` under `serde`.

#### Construction

```rust
impl From<u64> for VectorId;          // infallible
impl TryFrom<Vec<u8>> for VectorId;   // rejects an empty key
```

- **`From<u64>`** builds a `U64`.
- **`TryFrom<Vec<u8>>`** builds a `Bytes`, returning [`Err(IqdbError::InvalidConfig { reason })`](#iqdberror) for an empty key — an empty identifier is a configuration-shape problem, not a malformed vector.

```rust
use iqdb_types::VectorId;

let a = VectorId::from(7u64);
assert_eq!(a, VectorId::U64(7));

let b = VectorId::try_from(vec![0xde, 0xad]).unwrap();
assert_eq!(b, VectorId::Bytes(vec![0xde, 0xad].into_boxed_slice()));

assert!(VectorId::try_from(Vec::new()).is_err()); // empty key rejected
```

#### `Display`

`U64` renders as the decimal integer; `Bytes` renders as lowercase hex — two chars per byte, no `0x` prefix, no separators (so a 32-byte hash renders as 64 hex chars). `Debug` keeps the `Bytes([...])` shape for source troubleshooting; `Display` is what belongs in operator-facing logs.

```rust
use iqdb_types::VectorId;

assert_eq!(VectorId::from(42u64).to_string(), "42");

let key = VectorId::try_from(vec![0x00, 0x0f, 0xf0]).unwrap();
assert_eq!(key.to_string(), "000ff0"); // each byte padded to two chars
```

#### Use as a map key

Because `VectorId` is `Eq + Hash`, it is a `HashMap`/`HashSet` key directly.

```rust
use std::collections::HashMap;
use iqdb_types::VectorId;

let mut store = HashMap::new();
store.insert(VectorId::from(1u64), "first");
assert_eq!(store.get(&VectorId::U64(1)), Some(&"first"));
```

---

## Metadata

### `Value`

```rust
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}
```

A flat, JSON-like scalar — the value type stored in [`Metadata`] and compared by [`Filter`]. Deliberately non-nested: metadata is a flat map of scalars, which keeps filtering simple and predictable. Because it holds an `f64`, it is `PartialEq` but **not** `Eq`.

**Derives / traits:** `Debug`, `Clone`, `PartialEq`; `Serialize`/`Deserialize` under `serde`.

```rust
use iqdb_types::Value;

let title = Value::String("intro".to_string());
let year = Value::Int(2026);
let score = Value::Float(0.5);
let flag = Value::Bool(true);
let none = Value::Null;

assert_eq!(year, Value::Int(2026));
assert_ne!(title, none);
let _ = (score, flag);
```

### `Metadata`

```rust
pub struct Metadata(/* private: BTreeMap<String, Value> */);
```

An **immutable, ordered** map of string keys to scalar [`Value`]s — the structured attributes a query filters on. `BTreeMap`-backed, so iteration is always in ascending key order; that determinism makes serde round-trips and test assertions stable across runs and machines. There are no setters — to change metadata, build a new value.

**Derives / traits:** `Debug`, `Clone`, `Default`, `PartialEq`; `Serialize`/`Deserialize` under `serde`.

#### Construction

```rust
impl From<BTreeMap<String, Value>> for Metadata;
impl FromIterator<(String, Value)> for Metadata; // lets you `.collect()`
```

```rust
use std::collections::BTreeMap;
use iqdb_types::{Metadata, Value};

// From an iterator of pairs (the common path).
let a: Metadata = [
    ("year".to_string(), Value::Int(2026)),
    ("ok".to_string(), Value::Bool(true)),
]
.into_iter()
.collect();
assert_eq!(a.len(), 2);

// From an existing BTreeMap.
let mut map = BTreeMap::new();
map.insert("k".to_string(), Value::Null);
let b = Metadata::from(map);
assert_eq!(b.get("k"), Some(&Value::Null));

// Empty default.
assert!(Metadata::default().is_empty());
```

#### Accessors

```rust
pub fn get(&self, key: &str) -> Option<&Value>;  // None if absent
pub fn len(&self) -> usize;
pub fn is_empty(&self) -> bool;
pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)>; // ascending key order
```

```rust
use iqdb_types::{Metadata, Value};

let meta: Metadata = [
    ("b".to_string(), Value::Int(2)),
    ("a".to_string(), Value::Int(1)),
]
.into_iter()
.collect();

assert_eq!(meta.get("a"), Some(&Value::Int(1)));
assert_eq!(meta.get("missing"), None);

// Always key-ordered, regardless of insertion order.
let keys: Vec<&String> = meta.iter().map(|(k, _)| k).collect();
assert_eq!(keys, ["a", "b"]);
```

---

## Distance

### `DistanceMetric`

```rust
#[non_exhaustive]
pub enum DistanceMetric {
    Cosine,
    DotProduct,
    Euclidean,
    Manhattan,
    Hamming,
}
```

The metric used to compare two vectors. This crate carries only the **tag** — the kernels that compute it live in `iqdb-distance`. Which metric is valid depends on how the vectors were produced: cosine/dot suit (normalized) embeddings, Euclidean/Manhattan suit raw coordinates, Hamming suits binary codes.

> **`#[non_exhaustive]`:** future releases may add metrics (e.g. Jaccard, Chebyshev) without it being a breaking change, so a `match` on `DistanceMetric` from another crate must include a wildcard `_` arm.

**Derives / traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`; `Serialize`/`Deserialize` under `serde`.

```rust
use iqdb_types::DistanceMetric;

let m = DistanceMetric::Cosine;
assert_eq!(m, DistanceMetric::Cosine);
assert_ne!(m, DistanceMetric::Euclidean);

// `Copy` + `Eq + Hash`: usable as a key or in a set.
use std::collections::HashSet;
let set: HashSet<_> = [DistanceMetric::Cosine, DistanceMetric::Hamming].into_iter().collect();
assert!(set.contains(&DistanceMetric::Cosine));
```

---

## Filters

### `Filter`

```rust
pub enum Filter {
    Eq  { field: String, value: Value },
    Neq { field: String, value: Value },
    Lt  { field: String, value: Value },
    Lte { field: String, value: Value },
    Gt  { field: String, value: Value },
    Gte { field: String, value: Value },
    In  { field: String, values: Vec<Value> },
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
}
```

A boolean expression tree over a record's [`Metadata`]: leaf comparisons combined with `And`/`Or`/`Not`. It describes *which* records a search may consider; evaluation lives in the engine. Build trees with the constructor helpers rather than the variants directly.

**Derives / traits:** `Debug`, `Clone`, `PartialEq`; `Serialize`/`Deserialize` under `serde`.

#### Leaf constructors

Each takes **`field`** (`impl Into<String>` — `&str` or `String`) and a **`value`** ([`Value`]); `is_in` takes **`values`** (`Vec<Value>`).

```rust
pub fn eq   (field: impl Into<String>, value: Value) -> Filter;
pub fn neq  (field: impl Into<String>, value: Value) -> Filter;
pub fn lt   (field: impl Into<String>, value: Value) -> Filter;
pub fn lte  (field: impl Into<String>, value: Value) -> Filter;
pub fn gt   (field: impl Into<String>, value: Value) -> Filter;
pub fn gte  (field: impl Into<String>, value: Value) -> Filter;
pub fn is_in(field: impl Into<String>, values: Vec<Value>) -> Filter;
```

```rust
use iqdb_types::{Filter, Value};

let f = Filter::eq("year", Value::Int(2026));
assert_eq!(f, Filter::Eq { field: "year".to_string(), value: Value::Int(2026) });

let any_of = Filter::is_in("year", vec![Value::Int(2025), Value::Int(2026)]);
assert!(matches!(any_of, Filter::In { .. }));
```

#### Combinators

```rust
pub fn and(filters: Vec<Filter>) -> Filter; // every sub-filter must match
pub fn or (filters: Vec<Filter>) -> Filter; // any sub-filter may match
pub fn not(inner: Filter)        -> Filter; // negate the sub-filter
```

- **Empty `and(vec![])`** evaluates to **`true`** (vacuous truth) — a "match everything" filter.
- **Empty `or(vec![])`** evaluates to **`false`** — a "match nothing" filter.

```rust
use iqdb_types::{Filter, Value};

// published == true AND year >= 1800
let f = Filter::and(vec![
    Filter::eq("published", Value::Bool(true)),
    Filter::gte("year", Value::Int(1800)),
]);
assert!(matches!(f, Filter::And(_)));
```

#### Null / absent-field semantics (important)

`Filter` is **closed-world**: a leaf comparison whose `field` is absent from a record evaluates to **`false`** — and so does a type mismatch (an `Int` field compared to a `String` literal), and any ordered comparison against `Value::Float(NaN)`. This makes `neq` and `not(eq)` **not** interchangeable on absent fields:

```rust
use iqdb_types::{Filter, Value};

// FALSE for a record with no `author` field (only matches a present, non-"ada" author):
let strict = Filter::neq("author", Value::String("ada".to_string()));

// TRUE for a record with no `author` field (the idiom for "missing OR non-matching"):
let inclusive = Filter::not(Filter::eq("author", Value::String("ada".to_string())));

assert!(matches!(strict, Filter::Neq { .. }));
assert!(matches!(inclusive, Filter::Not(_)));
```

---

## Search

### `SearchParams`

```rust
pub struct SearchParams {
    pub k: usize,
    pub ef: Option<usize>,
    pub metric: DistanceMetric,
    pub filter: Option<Filter>,
}
```

The parameters of a nearest-neighbour search.

- **`k`** — how many results to return.
- **`ef`** — optional search-breadth knob (the candidate-list size some approximate indexes expose, e.g. HNSW `efSearch`); `None` lets the engine choose, and exact search ignores it.
- **`metric`** — how distance is measured ([`DistanceMetric`]).
- **`filter`** — optional metadata predicate restricting which records are eligible.

**Derives / traits:** `Debug`, `Clone`, `PartialEq`; `Serialize`/`Deserialize` under `serde`.

#### `SearchParams::new`

```rust
pub fn new(k: usize, metric: DistanceMetric) -> SearchParams;
```

Tier-1 constructor: a top-`k` search under `metric`, with `ef = None` and `filter = None`. Layer the optional fields on with struct-update syntax.

```rust
use iqdb_types::{DistanceMetric, Filter, SearchParams, Value};

// Tier 1 — the common case.
let simple = SearchParams::new(10, DistanceMetric::Cosine);
assert_eq!(simple.k, 10);
assert_eq!(simple.ef, None);
assert!(simple.filter.is_none());

// Tier 2 — tune the optional knobs.
let tuned = SearchParams {
    ef: Some(128),
    filter: Some(Filter::eq("published", Value::Bool(true))),
    ..SearchParams::new(10, DistanceMetric::Euclidean)
};
assert_eq!(tuned.ef, Some(128));
assert!(tuned.filter.is_some());
```

### `Hit`

```rust
pub struct Hit {
    pub id: VectorId,
    pub distance: f32,
    pub metadata: Option<Metadata>,
}
```

One result of a search.

- **`id`** — the matched vector's [`VectorId`].
- **`distance`** — its distance from the query under the search's metric (**smaller is nearer**).
- **`metadata`** — the record's [`Metadata`], present only when the search was asked to return it.

**Derives / traits:** `Debug`, `Clone`, `PartialEq`; `Serialize`/`Deserialize` under `serde`.

#### `Hit::new`

```rust
pub fn new(id: VectorId, distance: f32) -> Hit;
```

Builds a hit with no metadata attached. Set the `metadata` field to attach it.

```rust
use iqdb_types::{Hit, Metadata, Value, VectorId};

let bare = Hit::new(VectorId::from(42u64), 0.125);
assert_eq!(bare.id, VectorId::U64(42));
assert_eq!(bare.distance, 0.125);
assert!(bare.metadata.is_none());

// Attach metadata via struct-update.
let meta: Metadata = [("title".to_string(), Value::String("intro".to_string()))]
    .into_iter()
    .collect();
let rich = Hit { metadata: Some(meta), ..Hit::new(VectorId::from(42u64), 0.125) };
assert!(rich.metadata.is_some());
```

---

## Errors

### `IqdbError`

```rust
#[non_exhaustive]
pub enum IqdbError {
    DimensionMismatch { expected: usize, found: usize },
    InvalidVector,
    InvalidConfig { reason: &'static str },
    NotFound,
    Duplicate,
    InvalidMetric,
    InvalidFilter,
    ResourceLimitExceeded { kind: &'static str, max: usize, found: usize },
}
```

The single domain error for the iqdb spine. Each variant names one specific failure, so a caller reacts to the cause rather than parsing a message. It implements [`error_forge::ForgeError`], so it slots into the portfolio error stack (`kind()`, `caption()`).

**Derives / traits:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`; `Display`; `std::error::Error`; `error_forge::ForgeError`. The enum stays `Copy` — that is why the string fields are `&'static str`, not `String`.

> **`#[non_exhaustive]`:** a `match` on `IqdbError` from another crate **must** include a wildcard `_` arm; future releases may add variants without it being a breaking change.

#### Variants

| Variant | Fields | Meaning |
|---|---|---|
| `DimensionMismatch` | `expected`, `found` | A vector's dimensionality did not match what the operation required. |
| `InvalidVector` | — | A vector was empty or held a non-finite component. |
| `InvalidConfig` | `reason` | A configuration value could not describe a working index/query; `reason` names which check failed. |
| `NotFound` | — | The requested id/record does not exist. |
| `Duplicate` | — | An insert collided with an id already present. |
| `InvalidMetric` | — | The distance metric was not valid for the operation/vectors. |
| `InvalidFilter` | — | A filter expression was malformed or could not be evaluated. |
| `ResourceLimitExceeded` | `kind`, `max`, `found` | An input exceeded a configured cap; `kind` names the cap. Surfaces from the `Database` write boundary — the constructors in this crate never produce it. |

#### `Display`, `kind()`, `caption()`

`Display` is the full operator-facing message (with details); `kind()` is a stable machine identifier (the variant name); `caption()` is a fixed human summary, distinct per variant.

```rust
use error_forge::ForgeError;
use iqdb_types::IqdbError;

let err = IqdbError::DimensionMismatch { expected: 768, found: 384 };
assert_eq!(err.to_string(), "vector dimension mismatch: expected 768, found 384");
assert_eq!(err.kind(), "DimensionMismatch");
assert_eq!(err.caption(), "vector dimension does not match the index");

let cfg = IqdbError::InvalidConfig { reason: "dim must be greater than zero" };
assert_eq!(cfg.to_string(), "invalid configuration: dim must be greater than zero");
```

#### Branching on the cause

```rust
use iqdb_types::IqdbError;

fn describe(err: IqdbError) -> String {
    match err {
        IqdbError::DimensionMismatch { expected, found } => {
            format!("re-embed at {expected} dims (got {found})")
        }
        IqdbError::NotFound => "no such record".to_string(),
        // Required: the enum is #[non_exhaustive].
        other => other.to_string(),
    }
}

assert_eq!(
    describe(IqdbError::DimensionMismatch { expected: 3, found: 2 }),
    "re-embed at 3 dims (got 2)",
);
```

### `Result`

```rust
pub type Result<T> = core::result::Result<T, IqdbError>;
```

The crate-wide result alias — every fallible API returns it.

```rust
use iqdb_types::{IqdbError, Result};

fn require_non_empty(dim: usize) -> Result<()> {
    if dim == 0 {
        return Err(IqdbError::InvalidConfig { reason: "dim must be greater than zero" });
    }
    Ok(())
}

assert!(require_non_empty(3).is_ok());
assert!(require_non_empty(0).is_err());
```

---

## Feature flags

| Feature | Default | Effect |
|---|---|---|
| `serde` | off | Derives `Serialize`/`Deserialize` on every public type. `VectorRef` is `Serialize`-only (a borrowed view has nowhere to own decoded data). |
| `testing` | off | Exposes [`Vector::new_unchecked`], a test-only escape hatch for building a `Vector` without validation. A production build cannot reach it. |

The default build pulls only `error-forge`. Enabling `serde` additionally pulls the `serde` crate; `serde_json` stays a dev-dependency.

---

## Trait implementation matrix

| Type | `Copy` | `Eq` / `Hash` | `Display` | `Default` | serde |
|---|:---:|:---:|:---:|:---:|:---:|
| `Vector` | — | — | — | — | `Serialize` + `Deserialize` |
| `VectorRef<'a>` | ✅ | — | — | — | `Serialize` only |
| `VectorId` | — | ✅ | ✅ | — | `Serialize` + `Deserialize` |
| `Value` | — | — | — | — | `Serialize` + `Deserialize` |
| `Metadata` | — | — | — | ✅ | `Serialize` + `Deserialize` |
| `DistanceMetric` | ✅ | ✅ | — | — | `Serialize` + `Deserialize` |
| `Filter` | — | — | — | — | `Serialize` + `Deserialize` |
| `SearchParams` | — | — | — | — | `Serialize` + `Deserialize` |
| `Hit` | — | — | — | — | `Serialize` + `Deserialize` |
| `IqdbError` | ✅ | ✅ | ✅ | — | — |

All types implement `Debug`, `Clone`, and `PartialEq`.

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>

# iqdb-types -- Roadmap

> **Status: 1.0.0 reached (stable).** This file is retained as the record of the
> path from scaffold to a stable 1.0 — every phase below is DONE. The frozen 1.x
> public API is recorded under v0.5.0. Each phase had hard exit criteria; the
> hard parts were front-loaded, not deferred.
>
> **Anti-deferral rule:** no phase pushed a listed hard task to a later phase unless this file records the move and the reason.

---

## v0.1.0 -- Scaffold (DONE)

Compiles, CI green, structure correct, no domain logic.

Exit criteria:
- [x] Manifest, README, CHANGELOG, REPS, license, CI, deny, clippy, rustfmt in place.
- [x] Public API surface sketched in `docs/API.md`.

---

## v0.2.0 -- Core types (THE HARD PART, NOT DEFERRED) (DONE)

In scope: `Vector`/`VectorRef`, `VectorId`, `Distance`, `Hit`, `Metadata`, `SearchParams`; their `Display`/`Default`/ordering/equality impls. This API is reviewed hard because it cascades through the whole family.
Exit criteria:
- [x] Every public item has rustdoc + runnable example.
- [x] Type contracts (ordering, equality, conversions) are property-tested (delivered in 0.3.0 via `tests/properties.rs`).
- [x] Compiles fast with no heavy deps. _(The crate uses the REPS deny-style lint header rather than `#![forbid(unsafe_code)]`; it contains no unsafe.)_

---

## v0.3.0 -- serde + ergonomics (DONE)

In scope: `serde` support for every public type under the feature; constructors and conversion ergonomics.

> `serde` support and the fallible `Vector::new`/`TryFrom` construction
> ergonomics shipped early, in the 0.2.0 release; the property-test coverage
> landed in 0.3.0.

Exit criteria:
- [x] serde round-trip property test for every public type (`tests/properties.rs`).
- [x] Borrowed/owned conversions covered (`Vector` ↔ `Vec<f32>`, `VectorRef` over `&[f32]`).

---

## v0.4.0 -- Feature freeze (DONE)

In scope: finalize the surface against early consumers (`iqdb-distance`, an index crate); `examples/`.
Exit criteria:
- [x] At least two downstream iQDB crates compile against this unchanged. _Satisfied by `tests/consumer_simulation.rs`: working analogues of `iqdb-distance`, an index crate, and `iqdb-filter` are built from the public surface at the **exact signatures the real Cortex crates expose**, cross-checked against those implementations and their roadmaps._
- [x] No `todo!`/`unimplemented!`. **Feature freeze declared:** the public type set is complete; no new types or methods will be added before 1.0. The one remaining API decision for v0.5.0 is whether `VectorRef` stays (real consumers pass `&[f32]` directly).

---

## v0.5.0 -- Hardening + API freeze (handle with extra care) (DONE)

In scope: final API review; cross-platform verification; `docs/API.md` complete. Because every other iQDB crate depends on this, the freeze is deliberate and conservative.
Exit criteria:
- [x] Public API frozen for 1.x (recorded below). `cargo audit` + `cargo deny` clean.
- [x] Cross-platform verification: Windows + Linux (WSL2 Ubuntu) on stable and the 1.87 MSRV; macOS via CI.

### Frozen public API (1.x) — recorded at v0.5.0

The following surface is frozen for the 1.x series. Additive, non-breaking changes (new methods, new variants on `#[non_exhaustive]` enums) remain allowed; anything else waits for 2.0.

- **Constants:** `VERSION: &str`.
- **`Vector`** (`Box<[f32]>`-backed, validated): `new`, `new_unchecked` (feature `testing`), `as_slice`, `len`, `is_empty`, `dim`, `into_inner`; `TryFrom<Vec<f32>>`.
- **`VectorRef<'a>`**: `as_slice`, `len`, `is_empty`, `dim`, `into_inner`; `From<&'a [f32]>`. (Retained — zero-copy borrowed view for the future Database/RAG/query layers.)
- **`VectorId`**: variants `U64(u64)`, `Bytes(Box<[u8]>)`; `From<u64>`, `TryFrom<Vec<u8>>`, `Display`.
- **`Value`** (exhaustive): `String`, `Int(i64)`, `Float(f64)`, `Bool(bool)`, `Null`.
- **`Metadata`**: `get`, `len`, `is_empty`, `iter`; `From<BTreeMap<String, Value>>`, `FromIterator<(String, Value)>`.
- **`DistanceMetric`** (`#[non_exhaustive]`): `Cosine`, `DotProduct`, `Euclidean`, `Manhattan`, `Hamming`.
- **`Filter`** (exhaustive): leaves `Eq`/`Neq`/`Lt`/`Lte`/`Gt`/`Gte`/`In` + `And`/`Or`/`Not`; constructors `eq`/`neq`/`lt`/`lte`/`gt`/`gte`/`is_in`/`and`/`or`/`not`.
- **`SearchParams`** (public fields `k`, `ef`, `metric`, `filter`): `new(k, metric)`.
- **`Hit`** (public fields `id`, `distance`, `metadata`): `new(id, distance)`.
- **`IqdbError`** (`#[non_exhaustive]`) + **`Result<T>`**.
- **Features:** `serde`, `testing`.

Deliberate freeze decisions:
- `DistanceMetric` and `IqdbError` are `#[non_exhaustive]` so the metric/error sets can grow without a break; `Value` and `Filter` stay exhaustive so consumers can match them fully.
- `SearchParams` and `Hit` are plain structs with public fields, keeping the Tier-1/Tier-2 struct-update ergonomics. Per-index tuning knobs (e.g. IVF `nprobe`, PQ rerank) live in each index crate's own config, **not** in the shared `SearchParams`, so this struct is expected to remain stable.

---

## v0.6.0 -> v0.9.x -- Alpha / Beta -> RC (COMPRESSED)

> This phase existed to soak the frozen API against the first real downstream
> consumers and fix what they surfaced. Under the project's deliberate
> **types-first ordering** (iqdb-types reaches 1.0 before any dependent crate is
> built), there are no live consumers to soak against yet. Its intent was met
> instead by `tests/consumer_simulation.rs`, which exercises the public surface
> at the **exact signatures** the real `iqdb-distance`, index, and `iqdb-filter`
> crates expose (cross-checked against those Cortex implementations). With the
> API frozen, every Definition-of-Done criterion met, and cross-platform + MSRV
> verification green, this phase is compressed and the crate proceeds to 1.0.0.
> Any friction surfaced when the real consumers are merged will be an additive
> `1.0.x` change (the realistic gaps are additive; the `#[non_exhaustive]` enums
> absorb new metrics/errors without a break).

---

## v1.0.0 -- Stable (DONE)

Exit criteria:
- [x] Definition of Done (DIRECTIVES section 7) fully satisfied.
- [x] Public API frozen until 2.0 (recorded under v0.5.0 above).
- [x] Release note written; tag pushed. (Publish to crates.io at the maintainer's discretion.)

---

## Out of scope for 1.0 (record, do not drift into)

- Any index, distance math, or storage logic -- those are downstream iQDB crates.
- Database behavior -- the `iqdb` crate composes the family.
- Distributed/sharding types -- reserved for the distributed iQDB phase.

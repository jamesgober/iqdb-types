# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [1.0.0] - 2026-06-05

First stable release. The public API frozen at 0.5.0 is now committed under SemVer for the 1.x series: no breaking changes until 2.0. Every Definition-of-Done criterion (`dev/DIRECTIVES.md` §7) is satisfied, and the surface is verified on Windows and Linux across the stable and 1.87 MSRV toolchains.

### Added

- Edge-case test suite (`tests/edge_cases.rs`): `Value` NaN/cross-variant
  inequality, `Metadata` duplicate-key-last-wins and empty default, `Vector`
  single/large dimensions and finite extremes/subnormals, `VectorRef` empty
  view, `VectorId` single-byte/max-`u64` rendering, empty `Filter` combinators,
  `Hit` negative/zero distance, `IqdbError` `Copy`/equality, and `serde`
  null/empty-metadata and `VectorRef`-serialize behaviour.
- Empty-`And`/`Or` evaluation test in the consumer simulation, pinning the
  documented vacuous-truth (`And`) and false (`Or`) semantics.
- Expanded benchmark suite (`benches/types.rs`): added a `VectorId` `Display`
  (lowercase-hex) benchmark across UUID/SHA-256/long key sizes, alongside the
  `Vector::new` benchmark (the bench target was renamed `vector_new` → `types`).

### Changed

- Declared **1.0 stable**: the frozen surface (recorded in `dev/ROADMAP.md`) is
  now under the SemVer 1.x compatibility guarantee. The **public API is
  unchanged from 0.5.0** — this release adds only tests, benchmarks, and
  documentation polish on top of the stability commitment.

---

## [0.5.0] - 2026-06-05

API freeze. The public API is locked for the 1.x series — the frozen surface is recorded in `dev/ROADMAP.md`. Additive, non-breaking changes remain allowed; anything else waits for 2.0.

### Changed

- Public API declared frozen for 1.x (recorded in `dev/ROADMAP.md`). Freeze
  decisions: `VectorRef` retained; `DistanceMetric` and `IqdbError` are
  `#[non_exhaustive]` for forward compatibility while `Value` and `Filter`
  remain exhaustive; `SearchParams`/`Hit` stay plain public-field structs, with
  per-index tuning knobs reserved to the index crates rather than the shared
  `SearchParams`.
- Cross-platform verification extended to Linux (WSL2 Ubuntu) on both stable and
  the 1.87 MSRV, alongside Windows; macOS via CI.

---

## [0.4.0] - 2026-06-05

> **Note:** 0.4.0 was not tagged or published separately — its changes shipped as part of the [0.5.0] release. This section is retained as the record of the feature-freeze milestone.

Feature freeze. The public type set is complete and declared frozen — no new types or methods will be added before 1.0. This release adds the consumer-side proof that the surface is sufficient for the family.

### Added

- Consumer-simulation integration suite (`tests/consumer_simulation.rs`)
  satisfying the feature-freeze gate: minimal working analogues of
  `iqdb-distance`, an index crate, and `iqdb-filter` — built only from the
  public surface and at the **exact signatures the real crates expose**
  (`compute(metric, &[f32], &[f32])`; an `Arc<[f32]>`-storing index with
  `search(&[f32], &SearchParams)` and `delete(&VectorId)`; closed-world
  `Filter` evaluation). Cross-checked against the Cortex implementations and
  their roadmaps; confirms the vocabulary is sufficient and ergonomic.

### Fixed

- `tests/properties.rs`: bind the expected `Value` to a local before
  `prop_assert_eq!` so the metadata-lookup property compiles on the 1.87 MSRV
  (the inline temporary `Some(&Value::Int(..))` was dropped while borrowed —
  E0716 — under 1.87, though accepted by newer toolchains).

---

## [0.3.0] - 2026-06-05

Hardening and documentation release. No new public types — the v0.2.0 surface is now proven against generated inputs, fully documented and exampled, performance-tuned, and made forward-compatible where it matters.

### Added

- Property-based test suite (`tests/properties.rs`, `proptest` dev-dependency)
  covering the core type contracts per `dev/DIRECTIVES.md` §5/§8: `Vector`
  validation and round-trip, `VectorRef` viewing, `VectorId` `Display` (decimal
  / lowercase-hex) and construction, `Metadata` key ordering and lookup,
  `DistanceMetric` `Eq`/`Hash` consistency, and a `serde` JSON round-trip for
  every public type.
- `examples/` directory with runnable, documented examples: `vectors`,
  `metadata_and_filters`, `search`, `errors`, and `serde_roundtrip` (the last
  gated behind `required-features = ["serde"]`).
- `docs/API.md` rewritten as a complete reference: every public item with its
  description, parameters, trait/derive list, and multiple worked examples,
  plus a trait-implementation matrix.
- `benches/vector_new.rs` (`criterion` dev-dependency): a Criterion benchmark
  for the one measurable hot path, `Vector::new` validation, across common
  embedding dimensionalities (32 / 128 / 768 / 1024).

### Changed

- `Vector` now stores its components in a `Box<[f32]>` instead of a `Vec<f32>`.
  The value is immutable after construction, so it never needs spare capacity:
  the wrapper is one machine word smaller and the backing allocation is sized
  exactly to the data. The public API is unchanged — `Vector::new` still takes
  a `Vec<f32>` and `into_inner` still returns one (via an allocation-free
  `Box<[f32]>::into_vec`).
- Added `#[inline]` to the trivial accessors and constructors across the public
  surface so they inline across crate boundaries even when a consumer does not
  enable fat LTO.
- `DistanceMetric` is now `#[non_exhaustive]`, so future metrics can be added
  without a breaking change. Consumers matching on it must include a wildcard
  arm. (`IqdbError` is already `#[non_exhaustive]`; `Value` and `Filter` remain
  exhaustive so they can be matched fully.)

---

## [0.2.0] - 2026-06-05

First implementation release. The core public type surface lands — the shared vocabulary every other `iqdb-*` crate is built on — together with optional `serde` support.

### Added

- Initial public type surface for the iqdb vector-database spine:
  - `Vector` (owned) and `VectorRef<'a>` (borrowed) dense `f32` vectors.
  - `VectorId` — a `u64` or an opaque, non-empty byte key (`TryFrom<Vec<u8>>`
    rejects an empty key).
  - `Value` (a flat scalar) and the immutable, ordered `Metadata` map.
  - `DistanceMetric` — `Cosine`, `DotProduct`, `Euclidean`, `Manhattan`,
    `Hamming`.
  - `Filter` — a boolean expression tree (`Eq`/`Neq`/`Lt`/`Lte`/`Gt`/`Gte`/`In`
    leaves combined with `And`/`Or`/`Not`) with constructor helpers.
  - `SearchParams` and `Hit`.
  - `IqdbError` and `Result`, with `IqdbError` implementing
    `error_forge::ForgeError` and a distinct human-readable `caption()` per variant.
- `Vector::new(Vec<f32>) -> Result<Vector, IqdbError>` and
  `impl TryFrom<Vec<f32>> for Vector` — the fallible construction surface that
  rejects empty inputs and any non-finite component (NaN, ±∞) with
  `IqdbError::InvalidVector`. Validation happens at the type boundary (per
  `REPS.md`) so the math layer never has to defend against invalid floats.
- `Vector::new_unchecked` — test-only escape hatch exposed by the `testing`
  Cargo feature; production builds cannot reach it.
- `Filter` rustdoc pins the **closed-world** null-and-absent-field semantics:
  every leaf comparison on an absent field evaluates to `false`, so
  `Filter::neq("author", "ada")` does NOT match records lacking an `author`
  field — use `Filter::not(Filter::eq("author", "ada"))` for that.
- Optional `serde` feature deriving `Serialize`/`Deserialize` on the public
  types (`VectorRef` is `Serialize` only — a borrowed view has nowhere to own
  decoded data).
- `VERSION` constant exposing the crate's compile-time `CARGO_PKG_VERSION`.

### Changed

- MSRV set to `1.87` (Rust 2024 edition).

---

## [0.1.0] - 2026-05-30

Initial scaffold and repository bootstrap. No domain logic yet &mdash; this release establishes the structure, tooling, and quality gates the implementation is built on.

### Added

- `Cargo.toml` with crate metadata, Rust 2024 edition.
- Dual `Apache-2.0 OR MIT` license files.
- `README.md`, `CHANGELOG.md`, and a documentation skeleton.
- `REPS.md` compliance baseline.
- `.github/workflows/ci.yml` CI matrix; `deny.toml`, `clippy.toml`, `rustfmt.toml`.
- `dev/DIRECTIVES.md` and `dev/ROADMAP.md` (committed engineering standards + plan).

[Unreleased]: https://github.com/jamesgober/iqdb-types/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jamesgober/iqdb-types/compare/v0.5.0...v1.0.0
[0.5.0]: https://github.com/jamesgober/iqdb-types/compare/v0.3.0...v0.5.0
[0.3.0]: https://github.com/jamesgober/iqdb-types/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jamesgober/iqdb-types/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/iqdb-types/releases/tag/v0.1.0

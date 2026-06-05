<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>iqdb-types</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

### Added

### Changed

### Fixed

### Security

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

[Unreleased]: https://github.com/jamesgober/iqdb-types/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jamesgober/iqdb-types/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/iqdb-types/releases/tag/v0.1.0

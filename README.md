<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>iqdb-types</b>
    <br>
    <sub><sup>iQDB FOUNDATIONAL TYPES</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/iqdb-types"><img alt="Crates.io" src="https://img.shields.io/crates/v/iqdb-types"></a>
    <a href="https://crates.io/crates/iqdb-types"><img alt="Downloads" src="https://img.shields.io/crates/d/iqdb-types?color=%230099ff"></a>
    <a href="https://docs.rs/iqdb-types"><img alt="docs.rs" src="https://img.shields.io/docsrs/iqdb-types"></a>
    <a href="https://github.com/jamesgober/iqdb-types/actions"><img alt="CI" src="https://github.com/jamesgober/iqdb-types/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.87%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        <strong>iqdb-types</strong> defines the vocabulary the entire iQDB vector-database family speaks. When you read the docs for <code>iqdb-hnsw</code> or <code>iqdb-flat</code>, every type you meet &mdash; <code>Vector</code>, <code>VectorId</code>, <code>DistanceMetric</code>, <code>Filter</code>, <code>Hit</code>, <code>Metadata</code>, <code>SearchParams</code> &mdash; is defined here.
    </p>
    <p>
        It is deliberately the smallest, most stable crate in the family: almost pure type declarations, no iQDB-internal dependencies, and an optional <code>serde</code> feature. Because a breaking change here cascades through every other iQDB crate, the API is considered carefully before it freezes at 1.0 &mdash; that caution is the crate's whole job.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.87+</strong> (Rust 2024 edition). Pure types. Zero iQDB-internal deps. The most stable crate in the family.
    </p>
    <blockquote>
        <strong>Status: stable (1.0).</strong> The public API is committed under SemVer for the 1.x series — no breaking changes until 2.0. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

<h2>What it does</h2>

- **`Vector` / `VectorRef`** &mdash; owned (validated, `Box<[f32]>`-backed) and borrowed N-dimensional float vectors
- **`VectorId`** &mdash; stable identifier: 64-bit integer or user-supplied bytes
- **`DistanceMetric`** &mdash; metric tag: `Cosine`, `DotProduct`, `Euclidean`, `Manhattan`, `Hamming`
- **`Metadata` / `Value`** &mdash; immutable, ordered scalar key/value payload
- **`Filter`** &mdash; boolean expression tree over metadata (closed-world semantics)
- **`SearchParams`** &mdash; query parameters: k, metric, filter, search-time breadth
- **`Hit`** &mdash; a search result: id, distance, and optional metadata
- **`IqdbError`** &mdash; the domain error (`error-forge` `ForgeError`)
- **`serde` support** &mdash; every public type (de)serializes under the `serde` feature


<br>

## Installation

```toml
[dependencies]
iqdb-types = "1.0"
```

<br>

## Status

<code>v1.0.0</code> — **stable.** The public API is committed under SemVer for the 1.x series (no breaking changes until 2.0; the frozen surface is recorded in the <a href="./dev/ROADMAP.md"><code>ROADMAP</code></a>). It is property-tested across every invariant, fully documented with runnable examples and a complete <a href="./docs/API.md"><code>API reference</code></a>, performance-tuned (`Box<[f32]>`-backed `Vector`, inlined accessors, a benchmarked hot path), validated by a consumer-simulation suite mirroring the real downstream crates, and verified on Windows + Linux across stable and the 1.87 MSRV.

<hr>
<br>

## Where It Fits

`iqdb-types` is the root of the iQDB dependency graph. Everything builds on it:

- `iqdb-distance` &mdash; operates on these `Vector` and `DistanceMetric` types
- `iqdb-flat` / `iqdb-hnsw` / `iqdb-ivf` &mdash; index crates returning `Hit`s for `SearchParams`
- `iqdb` &mdash; the database, composing the family on this shared vocabulary

It has no first-party dependencies, so it is unblocked and buildable today.

<br>

## Contributing

See <a href="./dev/DIRECTIVES.md"><code>dev/DIRECTIVES.md</code></a> for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> &mdash; <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b> &mdash; <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>JAMES GOBER.</strong></sup>
</div>

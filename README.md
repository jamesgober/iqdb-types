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
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        <strong>iqdb-types</strong> defines the vocabulary the entire iQDB vector-database family speaks. When you read the docs for <code>iqdb-hnsw</code> or <code>iqdb-flat</code>, every type you meet &mdash; <code>Vector</code>, <code>VectorId</code>, <code>Distance</code>, <code>Hit</code>, <code>Metadata</code>, <code>SearchParams</code> &mdash; is defined here.
    </p>
    <p>
        It is deliberately the smallest, most stable crate in the family: almost pure type declarations, no iQDB-internal dependencies, and an optional <code>serde</code> feature. Because a breaking change here cascades through every other iQDB crate, the API is considered carefully before it freezes at 1.0 &mdash; that caution is the crate's whole job.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.85+</strong> (Rust 2024 edition). Pure types. Zero iQDB-internal deps. The most stable crate in the family.
    </p>
    <blockquote>
        <strong>Status: pre-1.0, in active development.</strong> The public API is being designed across the 0.x series and frozen at <code>1.0.0</code>. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

<h2>What it does</h2>

- **`Vector` / `VectorRef`** &mdash; owned and borrowed N-dimensional float vectors
- **`VectorId`** &mdash; stable identifier: 64-bit integer or user-supplied bytes
- **`Distance`** &mdash; distance/similarity metric tag (cosine, L2, dot, ...)
- **`Hit`** &mdash; a search result: id, distance, and optional metadata
- **`Metadata`** &mdash; structured key/value payload attached to a vector
- **`SearchParams`** &mdash; query parameters: k, metric, filters, search-time knobs
- **`serde` support** &mdash; every public type (de)serializes under the `serde` feature


<br>

## Installation

```toml
[dependencies]
iqdb-types = "0.1"
```

<br>

## Status

This is the <code>v0.1.0</code> scaffold: structure, tooling, and quality gates are in place; the implementation lands across the 0.x series per the <a href="./dev/ROADMAP.md"><code>ROADMAP</code></a> and <a href="./docs/API.md"><code>docs/API.md</code></a>.

<hr>
<br>

## Where It Fits

`iqdb-types` is the root of the iQDB dependency graph. Everything builds on it:

- `iqdb-distance` &mdash; operates on these `Vector` and `Distance` types
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

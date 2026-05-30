# iqdb-types -- Roadmap

> Path from scaffold to a stable 1.0. Each phase has hard exit criteria; the hard parts are front-loaded, not deferred.
>
> **Anti-deferral rule:** no phase pushes a listed hard task to a later phase unless this file is edited to record the move and the reason.

---

## v0.1.0 -- Scaffold (DONE)

Compiles, CI green, structure correct, no domain logic.

Exit criteria:
- [x] Manifest, README, CHANGELOG, REPS, license, CI, deny, clippy, rustfmt in place.
- [x] Public API surface sketched in `docs/API.md`.

---

## v0.2.0 -- Core types (THE HARD PART, NOT DEFERRED)

In scope: `Vector`/`VectorRef`, `VectorId`, `Distance`, `Hit`, `Metadata`, `SearchParams`; their `Display`/`Default`/ordering/equality impls. This API is reviewed hard because it cascades through the whole family.
Exit criteria:
- [ ] Every public item has rustdoc + runnable example.
- [ ] Type contracts (ordering, equality, conversions) are property-tested.
- [ ] `#![forbid(unsafe_code)]`; compiles fast with no heavy deps.

---

## v0.3.0 -- serde + ergonomics

In scope: `serde` support for every public type under the feature; constructors and conversion ergonomics.
Exit criteria:
- [ ] serde round-trip property test for every public type.
- [ ] Borrowed/owned conversions covered.

---

## v0.4.0 -- Feature freeze

In scope: finalize the surface against early consumers (`iqdb-distance`, an index crate); `examples/`.
Exit criteria:
- [ ] At least two downstream iQDB crates compile against this unchanged.
- [ ] No `todo!`/`unimplemented!`. Feature freeze declared.

---

## v0.5.0 -- Hardening + API freeze (handle with extra care)

In scope: final API review; cross-platform verification; `docs/API.md` complete. Because every other iQDB crate depends on this, the freeze is deliberate and conservative.
Exit criteria:
- [ ] Public API frozen for 1.x (recorded here). `cargo audit` + `cargo deny` clean.

---

## v0.6.0 -> v0.9.x -- Alpha / Beta -> RC

- 0.6.x-0.7.x: integrate against first real consumers; fix what they surface; MINOR-compatible additions only.
- 0.8.x (beta): bug fixes only; broader testing; final benchmarks captured.
- 0.9.x (rc): critical fixes + doc polish only.

---

## v1.0.0 -- Stable

Exit criteria:
- [ ] Definition of Done (DIRECTIVES section 7) fully satisfied.
- [ ] Public API frozen until 2.0.
- [ ] Release note written; published to crates.io; tag pushed.

---

## Out of scope for 1.0 (record, do not drift into)

- Any index, distance math, or storage logic -- those are downstream iQDB crates.
- Database behavior -- the `iqdb` crate composes the family.
- Distributed/sharding types -- reserved for the distributed iQDB phase.

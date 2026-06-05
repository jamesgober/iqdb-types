//! # iqdb-types
//!
//! Shared, dependency-light public types for the HiveDB **iqdb** vector-database
//! spine. This crate is pure data: the vectors that get indexed, the ids that
//! name them, the metadata attached to them, the metrics that compare them, the
//! parameters and filters of a search, the hits it returns, and the one domain
//! error that ties them together. It holds no engine, no storage, and no I/O —
//! it is the vocabulary every other `iqdb-*` crate shares so they agree on
//! shapes without depending on each other.
//!
//! Its only runtime dependency is [`error_forge`], whose `ForgeError` trait
//! [`IqdbError`] implements.
//!
//! ## Feature flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `serde` | no | Derives [`Serialize`](https://docs.rs/serde)/[`Deserialize`](https://docs.rs/serde) on the public types. [`VectorRef`] is `Serialize` only — a borrowed view has nowhere to own decoded data. |
//!
//! ## Example
//!
//! ```
//! use iqdb_types::{DistanceMetric, Filter, SearchParams, Value, Vector, VectorId};
//!
//! // An embedding to index, and an id that names it. `Vector::new`
//! // validates the contents up front (no empty, no NaN/Inf).
//! let embedding = Vector::new(vec![0.1, 0.2, 0.3]).unwrap();
//! let id = VectorId::from(1u64);
//!
//! // Query parameters: a top-3 cosine search, restricted to published records.
//! let params = SearchParams {
//!     filter: Some(Filter::eq("published", Value::Bool(true))),
//!     ..SearchParams::new(3, DistanceMetric::Cosine)
//! };
//!
//! assert_eq!(embedding.dim(), 3);
//! assert_eq!(id, VectorId::U64(1));
//! assert_eq!(params.k, 3);
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
#![deny(unused_results)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::unreachable)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod error;
mod filter;
mod hit;
mod id;
mod metadata;
mod metric;
mod search;
mod vector;

pub use crate::error::{IqdbError, Result};
pub use crate::filter::Filter;
pub use crate::hit::Hit;
pub use crate::id::VectorId;
pub use crate::metadata::{Metadata, Value};
pub use crate::metric::DistanceMetric;
pub use crate::search::SearchParams;
pub use crate::vector::{Vector, VectorRef};

/// The version of this crate, taken from `Cargo.toml` at compile time.
///
/// Exposed so a consumer can report the exact `iqdb-types` build it links
/// against — useful in diagnostics and version-skew checks across the iqdb
/// crate family.
///
/// # Examples
///
/// ```
/// // Carries a `major.minor.patch` SemVer core.
/// let version = iqdb_types::VERSION;
/// assert_eq!(version.split('.').count(), 3);
/// assert!(version.split('.').all(|part| !part.is_empty()));
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

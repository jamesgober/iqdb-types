//! The iqdb domain error.
//!
//! [`IqdbError`] is the single error type for the iqdb vector-database spine.
//! Each variant names one specific failure so a caller can react to the exact
//! cause rather than parsing a message. It implements
//! [`error_forge::ForgeError`], so it slots into the portfolio error stack
//! (kinds, captions, the central error hook) the same way every other domain
//! error does.

use core::fmt;

use error_forge::ForgeError;

/// An error from an iqdb vector-database operation.
///
/// Each variant identifies one specific failure. The enum is
/// `#[non_exhaustive]`: future releases may add variants without it being a
/// breaking change, so a `match` on it must include a wildcard arm.
///
/// # Examples
///
/// ```
/// use iqdb_types::IqdbError;
///
/// let err = IqdbError::DimensionMismatch { expected: 3, found: 2 };
/// assert_eq!(
///     err.to_string(),
///     "vector dimension mismatch: expected 3, found 2",
/// );
///
/// let cfg = IqdbError::InvalidConfig { reason: "dim must be greater than zero" };
/// assert_eq!(
///     cfg.to_string(),
///     "invalid configuration: dim must be greater than zero",
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IqdbError {
    /// A vector did not have the dimensionality the operation required —
    /// `expected` is the index's dimension, `found` is what was supplied.
    DimensionMismatch {
        /// The dimensionality the operation required.
        expected: usize,
        /// The dimensionality that was actually supplied.
        found: usize,
    },
    /// A vector was not valid for the operation (for example, empty when a
    /// non-empty vector was required).
    InvalidVector,
    /// A configuration value could not describe a working index or query.
    /// `reason` is a short static description of which configuration check
    /// failed, suitable for inclusion in operator-facing logs.
    InvalidConfig {
        /// Short static description of which configuration check failed.
        reason: &'static str,
    },
    /// The requested id or record does not exist.
    NotFound,
    /// An insert collided with an id that is already present.
    Duplicate,
    /// The distance metric was not valid for the operation or the vectors.
    InvalidMetric,
    /// A filter expression was malformed or could not be evaluated.
    InvalidFilter,
    /// An incoming write exceeded a configured resource limit. `kind` names
    /// which cap was hit (one of `"id_bytes"`, `"metadata_keys"`,
    /// `"metadata_key_bytes"`, `"metadata_value_string_bytes"`,
    /// `"total_vectors"`); `max` is the cap; `found` is what the caller
    /// supplied. Surfaces from the `Database` write boundary; the
    /// type-level constructors in this crate never produce it.
    ResourceLimitExceeded {
        /// Short, stable identifier for the cap that was exceeded.
        kind: &'static str,
        /// The configured maximum the cap allowed.
        max: usize,
        /// The value the caller actually supplied.
        found: usize,
    },
}

impl fmt::Display for IqdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionMismatch { expected, found } => {
                write!(
                    f,
                    "vector dimension mismatch: expected {expected}, found {found}"
                )
            }
            Self::InvalidVector => f.write_str("invalid vector"),
            Self::InvalidConfig { reason } => write!(f, "invalid configuration: {reason}"),
            Self::NotFound => f.write_str("not found"),
            Self::Duplicate => f.write_str("duplicate id"),
            Self::InvalidMetric => f.write_str("invalid distance metric"),
            Self::InvalidFilter => f.write_str("invalid filter"),
            Self::ResourceLimitExceeded { kind, max, found } => {
                write!(f, "resource limit exceeded: {kind} max={max} found={found}")
            }
        }
    }
}

impl std::error::Error for IqdbError {}

impl ForgeError for IqdbError {
    fn kind(&self) -> &'static str {
        match self {
            Self::DimensionMismatch { .. } => "DimensionMismatch",
            Self::InvalidVector => "InvalidVector",
            Self::InvalidConfig { .. } => "InvalidConfig",
            Self::NotFound => "NotFound",
            Self::Duplicate => "Duplicate",
            Self::InvalidMetric => "InvalidMetric",
            Self::InvalidFilter => "InvalidFilter",
            Self::ResourceLimitExceeded { .. } => "ResourceLimitExceeded",
        }
    }

    fn caption(&self) -> &'static str {
        match self {
            Self::DimensionMismatch { .. } => "vector dimension does not match the index",
            Self::InvalidVector => "vector is empty or contains non-finite components",
            Self::InvalidConfig { .. } => "configuration is not valid for the requested operation",
            Self::NotFound => "requested id is not present in the index",
            Self::Duplicate => "id already exists in the index",
            Self::InvalidMetric => "distance metric does not match the index or vectors",
            Self::InvalidFilter => "filter expression is malformed or cannot be evaluated",
            Self::ResourceLimitExceeded { .. } => "an input exceeded a configured resource limit",
        }
    }
}

/// A specialized [`Result`](core::result::Result) whose error is [`IqdbError`].
///
/// # Examples
///
/// ```
/// use iqdb_types::{IqdbError, Result};
///
/// fn require_non_empty(dim: usize) -> Result<()> {
///     if dim == 0 {
///         return Err(IqdbError::InvalidConfig { reason: "dim must be greater than zero" });
///     }
///     Ok(())
/// }
///
/// assert!(require_non_empty(0).is_err());
/// assert!(require_non_empty(3).is_ok());
/// ```
pub type Result<T> = core::result::Result<T, IqdbError>;

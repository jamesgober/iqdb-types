//! Stable identifiers for stored vectors.
//!
//! A [`VectorId`] is either a compact 64-bit integer or an opaque byte key, so
//! a caller can use whichever id space their data already has (a row id, a
//! content hash, a UUID's bytes) without the iqdb spine imposing one.

use core::fmt;

use crate::error::IqdbError;

/// A stable identifier for a stored vector.
///
/// Either a compact [`U64`](VectorId::U64) integer id or an opaque
/// [`Bytes`](VectorId::Bytes) key. Build a `U64` id with
/// [`From<u64>`](VectorId::from); build a `Bytes` id with
/// [`TryFrom<Vec<u8>>`](VectorId::try_from), which rejects an empty key.
///
/// # Examples
///
/// ```
/// use iqdb_types::VectorId;
///
/// let numeric = VectorId::from(7u64);
/// assert_eq!(numeric, VectorId::U64(7));
///
/// let key = VectorId::try_from(vec![0xde, 0xad]).expect("non-empty key");
/// assert_eq!(key, VectorId::Bytes(vec![0xde, 0xad].into_boxed_slice()));
///
/// // An empty byte key is rejected.
/// assert!(VectorId::try_from(Vec::new()).is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VectorId {
    /// A compact 64-bit integer id (for example, a row id).
    U64(u64),
    /// An opaque, non-empty byte key (for example, a content hash or a UUID's
    /// raw bytes).
    Bytes(Box<[u8]>),
}

impl From<u64> for VectorId {
    #[inline]
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl TryFrom<Vec<u8>> for VectorId {
    type Error = IqdbError;

    /// Builds a [`Bytes`](VectorId::Bytes) id, rejecting an empty key with
    /// [`IqdbError::InvalidConfig`] (a malformed identifier is a
    /// configuration shape problem, not a malformed vector).
    #[inline]
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(IqdbError::InvalidConfig {
                reason: "VectorId::Bytes key must not be empty",
            });
        }
        Ok(Self::Bytes(bytes.into_boxed_slice()))
    }
}

/// Operator-facing rendering: `U64` is the decimal integer; `Bytes` is
/// lowercase hex with no prefix and no separators (so a 32-byte
/// SHA-256-shaped id renders as 64 hex characters). `Debug` keeps the
/// `Bytes([...])` shape for in-source troubleshooting; `Display` is the
/// shape that lands in operator-facing logs and error messages.
///
/// # Examples
///
/// ```
/// use iqdb_types::VectorId;
///
/// assert_eq!(VectorId::from(7u64).to_string(), "7");
///
/// let key = VectorId::try_from(vec![0xde, 0xad, 0xbe, 0xef]).expect("non-empty");
/// assert_eq!(key.to_string(), "deadbeef");
/// ```
impl fmt::Display for VectorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U64(n) => write!(f, "{n}"),
            Self::Bytes(bytes) => {
                for byte in bytes.iter() {
                    write!(f, "{byte:02x}")?;
                }
                Ok(())
            }
        }
    }
}

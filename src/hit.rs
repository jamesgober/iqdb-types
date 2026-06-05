//! A single search result.

use crate::id::VectorId;
use crate::metadata::Metadata;

/// One result of a similarity search: a matched record and its distance.
///
/// `id` identifies the matched vector, `distance` is its distance from the
/// query under the search's metric (smaller is nearer), and `metadata` is the
/// record's metadata when the engine was asked to return it. Build a bare hit
/// with [`Hit::new`] and attach metadata by setting the field.
///
/// # Examples
///
/// ```
/// use iqdb_types::{Hit, VectorId};
///
/// let hit = Hit::new(VectorId::from(42u64), 0.125);
/// assert_eq!(hit.id, VectorId::U64(42));
/// assert_eq!(hit.distance, 0.125);
/// assert!(hit.metadata.is_none());
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hit {
    /// The identifier of the matched vector.
    pub id: VectorId,
    /// The distance from the query under the search's metric (smaller is
    /// nearer).
    pub distance: f32,
    /// The record's metadata, when the search was asked to return it.
    pub metadata: Option<Metadata>,
}

impl Hit {
    /// Creates a hit for `id` at `distance`, with no metadata attached.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Hit, VectorId};
    ///
    /// let hit = Hit::new(VectorId::from(1u64), 2.5);
    /// assert_eq!(hit.distance, 2.5);
    /// assert!(hit.metadata.is_none());
    /// ```
    #[must_use]
    pub fn new(id: VectorId, distance: f32) -> Self {
        Self {
            id,
            distance,
            metadata: None,
        }
    }
}

//! The distance metric used to compare vectors.

/// The metric used to measure distance (or similarity) between two vectors.
///
/// Which metric is valid depends on how the vectors were produced — cosine and
/// dot product suit normalized embeddings, the geometric metrics suit raw
/// coordinates, and Hamming suits binary codes. The engine selects the matching
/// comparison from this tag.
///
/// # Examples
///
/// ```
/// use iqdb_types::DistanceMetric;
///
/// let metric = DistanceMetric::Cosine;
/// assert_eq!(metric, DistanceMetric::Cosine);
/// assert_ne!(metric, DistanceMetric::Euclidean);
/// ```
///
/// The enum is `#[non_exhaustive]`: future releases may add metrics (for
/// example, Jaccard or Chebyshev) without it being a breaking change, so a
/// `match` on it from another crate must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistanceMetric {
    /// Cosine distance — the angle between vectors, ignoring magnitude. Suits
    /// normalized embeddings.
    Cosine,
    /// Dot-product similarity — magnitude-sensitive inner product. Suits
    /// embeddings where magnitude carries signal.
    DotProduct,
    /// Euclidean (L2) distance — straight-line distance between coordinates.
    Euclidean,
    /// Manhattan (L1) distance — sum of absolute per-component differences.
    Manhattan,
    /// Hamming distance — count of differing positions. Suits binary codes.
    Hamming,
}

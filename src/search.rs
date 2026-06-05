//! Parameters for a similarity search.

use crate::filter::Filter;
use crate::metric::DistanceMetric;

/// The parameters of a nearest-neighbor search.
///
/// `k` is how many results to return, `metric` is how distance is measured,
/// `ef` is an optional search-breadth knob (the candidate-list size some
/// approximate indexes expose; ignored by exact search), and `filter` is an
/// optional metadata predicate restricting which records are eligible. Start
/// from [`SearchParams::new`] and set the optional fields as needed.
///
/// # Examples
///
/// ```
/// use iqdb_types::{DistanceMetric, Filter, SearchParams, Value};
///
/// let params = SearchParams {
///     filter: Some(Filter::eq("published", Value::Bool(true))),
///     ..SearchParams::new(10, DistanceMetric::Cosine)
/// };
///
/// assert_eq!(params.k, 10);
/// assert_eq!(params.metric, DistanceMetric::Cosine);
/// assert_eq!(params.ef, None);
/// assert!(params.filter.is_some());
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchParams {
    /// The number of nearest neighbors to return.
    pub k: usize,
    /// Optional search breadth (candidate-list size) for approximate indexes;
    /// `None` lets the engine choose, and exact search ignores it.
    pub ef: Option<usize>,
    /// The distance metric used to rank candidates.
    pub metric: DistanceMetric,
    /// Optional metadata predicate restricting which records are eligible.
    pub filter: Option<Filter>,
}

impl SearchParams {
    /// Creates parameters for a top-`k` search under `metric`, with no search
    /// breadth and no filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{DistanceMetric, SearchParams};
    ///
    /// let params = SearchParams::new(5, DistanceMetric::Euclidean);
    /// assert_eq!(params.k, 5);
    /// assert_eq!(params.ef, None);
    /// assert!(params.filter.is_none());
    /// ```
    #[must_use]
    pub fn new(k: usize, metric: DistanceMetric) -> Self {
        Self {
            k,
            ef: None,
            metric,
            filter: None,
        }
    }
}

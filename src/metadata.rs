//! Scalar metadata attached to a stored vector.
//!
//! [`Metadata`] is an immutable, ordered map from string keys to scalar
//! [`Value`]s. It carries the structured attributes a query filters on (an
//! author, a timestamp encoded as an integer, a published flag). Construct it
//! once from a map or an iterator; it has no in-place mutators.

use std::collections::{BTreeMap, btree_map};

/// A scalar metadata value.
///
/// A closed set of JSON-like scalars. It deliberately has no nesting — metadata
/// is a flat map of scalars, which keeps filtering simple and predictable. It
/// holds an `f64`, so it is [`PartialEq`] but not [`Eq`].
///
/// # Examples
///
/// ```
/// use iqdb_types::Value;
///
/// let title = Value::String("intro".to_string());
/// let year = Value::Int(2026);
/// let score = Value::Float(0.5);
/// let published = Value::Bool(true);
/// let missing = Value::Null;
///
/// assert_eq!(year, Value::Int(2026));
/// assert_ne!(title, missing);
/// let _ = (score, published);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    /// A UTF-8 string value.
    String(String),
    /// A signed 64-bit integer value.
    Int(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
    /// The absence of a value.
    Null,
}

/// An immutable, ordered map of metadata keys to [`Value`]s.
///
/// Build one from a [`BTreeMap`] with [`From`], or collect it from an iterator
/// of `(String, Value)` pairs. Read it with [`get`](Metadata::get),
/// [`len`](Metadata::len), [`is_empty`](Metadata::is_empty), and
/// [`iter`](Metadata::iter). There are no setters — to change metadata, build a
/// new value.
///
/// # Examples
///
/// ```
/// use iqdb_types::{Metadata, Value};
///
/// let meta: Metadata = [
///     ("title".to_string(), Value::String("intro".to_string())),
///     ("year".to_string(), Value::Int(2026)),
/// ]
/// .into_iter()
/// .collect();
///
/// assert_eq!(meta.len(), 2);
/// assert_eq!(meta.get("year"), Some(&Value::Int(2026)));
/// assert_eq!(meta.get("missing"), None);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata(BTreeMap<String, Value>);

impl Metadata {
    /// Returns the value for `key`, or `None` if the key is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Metadata, Value};
    ///
    /// let meta: Metadata =
    ///     [("year".to_string(), Value::Int(2026))].into_iter().collect();
    /// assert_eq!(meta.get("year"), Some(&Value::Int(2026)));
    /// assert_eq!(meta.get("nope"), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    /// Returns the number of entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Metadata, Value};
    ///
    /// let meta: Metadata =
    ///     [("a".to_string(), Value::Null)].into_iter().collect();
    /// assert_eq!(meta.len(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Metadata;
    ///
    /// assert!(Metadata::default().is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the entries in key order.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Metadata, Value};
    ///
    /// let meta: Metadata = [
    ///     ("b".to_string(), Value::Int(2)),
    ///     ("a".to_string(), Value::Int(1)),
    /// ]
    /// .into_iter()
    /// .collect();
    ///
    /// // BTreeMap iterates in key order.
    /// let keys: Vec<&String> = meta.iter().map(|(key, _)| key).collect();
    /// assert_eq!(keys, vec!["a", "b"]);
    /// ```
    #[inline]
    pub fn iter(&self) -> btree_map::Iter<'_, String, Value> {
        self.0.iter()
    }
}

impl From<BTreeMap<String, Value>> for Metadata {
    #[inline]
    fn from(map: BTreeMap<String, Value>) -> Self {
        Self(map)
    }
}

impl FromIterator<(String, Value)> for Metadata {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (String, Value)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

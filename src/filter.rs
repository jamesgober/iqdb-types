//! Metadata filter expressions.
//!
//! A [`Filter`] is a small boolean expression tree over a record's
//! [`Metadata`](crate::Metadata): leaf comparisons against a field's
//! [`Value`](crate::Value), combined with `And`/`Or`/`Not`. It describes *which*
//! records a search may consider; evaluation lives in the engine. The
//! constructor helpers ([`Filter::eq`], [`Filter::and`], [`Filter::not`], …)
//! make a tree readable to build by hand.

use crate::metadata::Value;

/// A boolean filter expression over record metadata.
///
/// Leaves compare one `field` against a [`Value`]; `And`/`Or`/`Not` combine
/// sub-expressions. Build it with the constructor helpers rather than the
/// variants directly.
///
/// # Null and absent-field semantics
///
/// `Filter` follows a **closed-world** rule: a leaf comparison whose `field` is
/// absent from a record's metadata evaluates to `false`. This applies to every
/// leaf — [`Eq`](Filter::Eq), [`Neq`](Filter::Neq), [`Lt`](Filter::Lt),
/// [`Lte`](Filter::Lte), [`Gt`](Filter::Gt), [`Gte`](Filter::Gte), and
/// [`In`](Filter::In). Type mismatches between the field's stored value and
/// the literal also evaluate to `false` (a `Value::Int` field compared against
/// a `Value::String` literal does not match).
///
/// This makes `Neq` and `Not(Eq)` **not** interchangeable on absent fields:
///
/// - `Filter::neq("author", "ada")` evaluates to `false` for a record with
///   no `author` field. It only matches records that explicitly carry an
///   `author` field whose value is not `"ada"`.
/// - `Filter::not(Filter::eq("author", "ada"))` evaluates to `true` for a
///   record with no `author`. This is the idiom for "records without this
///   field, or with a non-matching value."
///
/// `Value::Float(NaN)` under `Lt`/`Lte`/`Gt`/`Gte` also evaluates to `false`
/// (IEEE-754 unordered).
///
/// An explicit `IsNull` / `Exists` leaf is intentionally not part of v0.1; the
/// `Not(Eq(...))` idiom covers the common case. It is tracked as a possible
/// additive variant for a later release.
///
/// # Examples
///
/// ```
/// use iqdb_types::{Filter, Value};
///
/// // author == "ada" AND NOT (year > 2000)
/// let filter = Filter::and(vec![
///     Filter::eq("author", Value::String("ada".to_string())),
///     Filter::not(Filter::gt("year", Value::Int(2000))),
/// ]);
///
/// assert_eq!(
///     filter,
///     Filter::And(vec![
///         Filter::Eq { field: "author".to_string(), value: Value::String("ada".to_string()) },
///         Filter::Not(Box::new(Filter::Gt { field: "year".to_string(), value: Value::Int(2000) })),
///     ]),
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Filter {
    /// Matches when `field` equals `value`.
    Eq {
        /// The metadata field to test.
        field: String,
        /// The value to compare against.
        value: Value,
    },
    /// Matches when `field` does not equal `value`.
    Neq {
        /// The metadata field to test.
        field: String,
        /// The value to compare against.
        value: Value,
    },
    /// Matches when `field` is strictly less than `value`.
    Lt {
        /// The metadata field to test.
        field: String,
        /// The value to compare against.
        value: Value,
    },
    /// Matches when `field` is less than or equal to `value`.
    Lte {
        /// The metadata field to test.
        field: String,
        /// The value to compare against.
        value: Value,
    },
    /// Matches when `field` is strictly greater than `value`.
    Gt {
        /// The metadata field to test.
        field: String,
        /// The value to compare against.
        value: Value,
    },
    /// Matches when `field` is greater than or equal to `value`.
    Gte {
        /// The metadata field to test.
        field: String,
        /// The value to compare against.
        value: Value,
    },
    /// Matches when `field` equals any of `values`.
    In {
        /// The metadata field to test.
        field: String,
        /// The set of acceptable values.
        values: Vec<Value>,
    },
    /// Matches when every sub-filter matches.
    And(Vec<Filter>),
    /// Matches when any sub-filter matches.
    Or(Vec<Filter>),
    /// Matches when the sub-filter does not match.
    Not(Box<Filter>),
}

impl Filter {
    /// Builds an [`Eq`](Filter::Eq) leaf: `field == value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::eq("year", Value::Int(2026));
    /// assert_eq!(f, Filter::Eq { field: "year".to_string(), value: Value::Int(2026) });
    /// ```
    #[must_use]
    pub fn eq(field: impl Into<String>, value: Value) -> Self {
        Self::Eq {
            field: field.into(),
            value,
        }
    }

    /// Builds a [`Neq`](Filter::Neq) leaf: `field != value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::neq("status", Value::String("draft".to_string()));
    /// assert!(matches!(f, Filter::Neq { .. }));
    /// ```
    #[must_use]
    pub fn neq(field: impl Into<String>, value: Value) -> Self {
        Self::Neq {
            field: field.into(),
            value,
        }
    }

    /// Builds an [`Lt`](Filter::Lt) leaf: `field < value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::lt("year", Value::Int(2000));
    /// assert!(matches!(f, Filter::Lt { .. }));
    /// ```
    #[must_use]
    pub fn lt(field: impl Into<String>, value: Value) -> Self {
        Self::Lt {
            field: field.into(),
            value,
        }
    }

    /// Builds an [`Lte`](Filter::Lte) leaf: `field <= value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::lte("year", Value::Int(2000));
    /// assert!(matches!(f, Filter::Lte { .. }));
    /// ```
    #[must_use]
    pub fn lte(field: impl Into<String>, value: Value) -> Self {
        Self::Lte {
            field: field.into(),
            value,
        }
    }

    /// Builds a [`Gt`](Filter::Gt) leaf: `field > value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::gt("year", Value::Int(2000));
    /// assert!(matches!(f, Filter::Gt { .. }));
    /// ```
    #[must_use]
    pub fn gt(field: impl Into<String>, value: Value) -> Self {
        Self::Gt {
            field: field.into(),
            value,
        }
    }

    /// Builds a [`Gte`](Filter::Gte) leaf: `field >= value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::gte("year", Value::Int(2000));
    /// assert!(matches!(f, Filter::Gte { .. }));
    /// ```
    #[must_use]
    pub fn gte(field: impl Into<String>, value: Value) -> Self {
        Self::Gte {
            field: field.into(),
            value,
        }
    }

    /// Builds an [`In`](Filter::In) leaf: `field` equals any of `values`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::is_in("year", vec![Value::Int(2025), Value::Int(2026)]);
    /// assert!(matches!(f, Filter::In { .. }));
    /// ```
    #[must_use]
    pub fn is_in(field: impl Into<String>, values: Vec<Value>) -> Self {
        Self::In {
            field: field.into(),
            values,
        }
    }

    /// Builds an [`And`](Filter::And) node: every sub-filter must match.
    ///
    /// An empty `filters` vector evaluates to **`true`** (vacuous truth —
    /// "every element of an empty set satisfies the predicate"). A caller
    /// using `Filter::and(vec![])` as a "match everything" filter is
    /// relying on documented behaviour.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::and(vec![
    ///     Filter::eq("a", Value::Bool(true)),
    ///     Filter::eq("b", Value::Bool(false)),
    /// ]);
    /// assert!(matches!(f, Filter::And(_)));
    /// ```
    #[must_use]
    pub fn and(filters: Vec<Filter>) -> Self {
        Self::And(filters)
    }

    /// Builds an [`Or`](Filter::Or) node: any sub-filter may match.
    ///
    /// An empty `filters` vector evaluates to **`false`** ("no element
    /// of an empty set satisfies the predicate"). A caller using
    /// `Filter::or(vec![])` as a "match nothing" filter is relying on
    /// documented behaviour.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::or(vec![
    ///     Filter::eq("a", Value::Bool(true)),
    ///     Filter::eq("b", Value::Bool(true)),
    /// ]);
    /// assert!(matches!(f, Filter::Or(_)));
    /// ```
    #[must_use]
    pub fn or(filters: Vec<Filter>) -> Self {
        Self::Or(filters)
    }

    /// Builds a [`Not`](Filter::Not) node: negates the sub-filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{Filter, Value};
    ///
    /// let f = Filter::not(Filter::eq("a", Value::Null));
    /// assert!(matches!(f, Filter::Not(_)));
    /// ```
    // Keep the builder-style `not(inner)` symmetric with `and`/`or`/`eq` rather
    // than implementing `std::ops::Not`, which negates `self` and reads wrong
    // for wrapping a sub-filter.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn not(inner: Filter) -> Self {
        Self::Not(Box::new(inner))
    }
}

//! Dense `f32` vectors: an owned [`Vector`] and a borrowed [`VectorRef`].
//!
//! These are the raw embeddings the iqdb spine indexes and searches. They are
//! thin, immutable wrappers — construct one from its data through the
//! fallible [`Vector::new`] (or the equivalent [`TryFrom<Vec<f32>>`] impl),
//! read it back through the accessors, and (for [`Vector`]) reclaim the
//! buffer with [`Vector::into_inner`]. There is no in-place mutation.

use crate::error::{IqdbError, Result};

/// An owned dense vector of `f32` components.
///
/// Construct one with the fallible [`Vector::new`] (which rejects empty
/// inputs and non-finite components) or its [`TryFrom<Vec<f32>>`] sibling;
/// read its components with [`as_slice`](Vector::as_slice) or reclaim the
/// buffer with [`into_inner`](Vector::into_inner).
///
/// Validation at this boundary keeps the rest of the spine free of input
/// checks: once a `Vector` exists, the math never has to defend against
/// empty, NaN, or infinite components.
///
/// # Examples
///
/// ```
/// use iqdb_types::{IqdbError, Vector};
///
/// let v = Vector::new(vec![1.0, 0.0, 0.0]).unwrap();
/// assert_eq!(v.dim(), 3);
/// assert_eq!(v.as_slice(), &[1.0, 0.0, 0.0]);
///
/// // Empty and non-finite inputs are rejected at construction.
/// assert_eq!(Vector::new(Vec::new()).unwrap_err(), IqdbError::InvalidVector);
/// assert_eq!(
///     Vector::new(vec![1.0, f32::NAN]).unwrap_err(),
///     IqdbError::InvalidVector,
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector(Vec<f32>);

impl Vector {
    /// Builds a `Vector` from `data`, validating the contents.
    ///
    /// Returns [`IqdbError::InvalidVector`] when:
    ///
    /// - `data` is empty, or
    /// - any component is not finite (NaN or ±infinity).
    ///
    /// Validating at the type boundary keeps the rest of the spine — and
    /// every consumer crate — free of input checks. Once a `Vector` is in
    /// hand the math can trust its contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::{IqdbError, Vector};
    ///
    /// let v = Vector::new(vec![0.1, 0.2, 0.3]).unwrap();
    /// assert_eq!(v.dim(), 3);
    ///
    /// assert_eq!(
    ///     Vector::new(Vec::new()).unwrap_err(),
    ///     IqdbError::InvalidVector,
    /// );
    /// assert_eq!(
    ///     Vector::new(vec![1.0, f32::INFINITY]).unwrap_err(),
    ///     IqdbError::InvalidVector,
    /// );
    /// ```
    pub fn new(data: Vec<f32>) -> Result<Self> {
        if data.is_empty() {
            return Err(IqdbError::InvalidVector);
        }
        if data.iter().any(|v| !v.is_finite()) {
            return Err(IqdbError::InvalidVector);
        }
        Ok(Self(data))
    }

    /// Builds a `Vector` from `data` without validating it.
    ///
    /// Available only when the crate is built with the `testing` feature.
    /// Production code MUST use [`Vector::new`] (or `TryFrom`); a production
    /// build of `iqdb-types` cannot compile a call to this constructor.
    ///
    /// Reserved for tests that deliberately need to construct otherwise-
    /// invalid vectors to assert downstream behavior on bad input.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "testing")]
    /// # {
    /// use iqdb_types::Vector;
    ///
    /// // Constructible only under the `testing` feature.
    /// let v = Vector::new_unchecked(vec![f32::NAN]);
    /// assert_eq!(v.len(), 1);
    /// # }
    /// ```
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn new_unchecked(data: Vec<f32>) -> Self {
        Self(data)
    }

    /// Borrows the components as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Vector;
    ///
    /// let v = Vector::new(vec![0.5, 0.5]).unwrap();
    /// assert_eq!(v.as_slice(), &[0.5, 0.5]);
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }

    /// Returns the number of components.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Vector;
    ///
    /// assert_eq!(Vector::new(vec![1.0, 2.0]).unwrap().len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the vector has no components.
    ///
    /// A `Vector` produced by [`Vector::new`] is never empty (empty inputs
    /// are rejected at construction); this method is `false` for every
    /// `Vector` outside the `testing`-gated `Vector::new_unchecked`.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Vector;
    ///
    /// assert!(!Vector::new(vec![1.0]).unwrap().is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the dimensionality of the vector (its component count).
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Vector;
    ///
    /// assert_eq!(Vector::new(vec![1.0, 2.0, 3.0]).unwrap().dim(), 3);
    /// ```
    #[must_use]
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    /// Consumes the vector and returns the underlying buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Vector;
    ///
    /// let v = Vector::new(vec![1.0, 2.0]).unwrap();
    /// assert_eq!(v.into_inner(), vec![1.0, 2.0]);
    /// ```
    #[must_use]
    pub fn into_inner(self) -> Vec<f32> {
        self.0
    }
}

impl TryFrom<Vec<f32>> for Vector {
    type Error = IqdbError;

    /// Delegates to [`Vector::new`]: rejects empty and non-finite inputs
    /// with [`IqdbError::InvalidVector`].
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::Vector;
    ///
    /// let v: Vector = vec![1.0, 0.0].try_into().unwrap();
    /// assert_eq!(v.dim(), 2);
    /// ```
    fn try_from(data: Vec<f32>) -> Result<Self> {
        Self::new(data)
    }
}

/// A borrowed dense vector of `f32` components.
///
/// A zero-copy view over a `&[f32]`, for passing query vectors without taking
/// ownership. It is [`Copy`]. With the `serde` feature it derives
/// [`Serialize`](https://docs.rs/serde) only — a borrowed view cannot be
/// deserialized into, since there is nowhere to own the decoded data.
///
/// # Examples
///
/// ```
/// use iqdb_types::VectorRef;
///
/// let data = [1.0, 0.0, 0.0];
/// let v = VectorRef::from(&data[..]);
/// assert_eq!(v.dim(), 3);
/// assert_eq!(v.as_slice(), &[1.0, 0.0, 0.0]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VectorRef<'a>(&'a [f32]);

impl<'a> VectorRef<'a> {
    /// Borrows the components as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::VectorRef;
    ///
    /// let data = [0.5, 0.5];
    /// assert_eq!(VectorRef::from(&data[..]).as_slice(), &[0.5, 0.5]);
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        self.0
    }

    /// Returns the number of components.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::VectorRef;
    ///
    /// let data = [1.0, 2.0];
    /// assert_eq!(VectorRef::from(&data[..]).len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the view has no components.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::VectorRef;
    ///
    /// let empty: [f32; 0] = [];
    /// assert!(VectorRef::from(&empty[..]).is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the dimensionality of the view (its component count).
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::VectorRef;
    ///
    /// let data = [1.0, 2.0, 3.0];
    /// assert_eq!(VectorRef::from(&data[..]).dim(), 3);
    /// ```
    #[must_use]
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    /// Returns the borrowed slice with its original lifetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use iqdb_types::VectorRef;
    ///
    /// let data = [1.0, 2.0];
    /// let v = VectorRef::from(&data[..]);
    /// let slice: &[f32] = v.into_inner();
    /// assert_eq!(slice, &[1.0, 2.0]);
    /// ```
    #[must_use]
    pub fn into_inner(self) -> &'a [f32] {
        self.0
    }
}

impl<'a> From<&'a [f32]> for VectorRef<'a> {
    fn from(data: &'a [f32]) -> Self {
        Self(data)
    }
}

use core::num::NonZeroUsize;

/// Represents a **nonzero** amount of bytes
/// that is required to perform some action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[must_use]
pub enum Size {
    /// The amount of bytes is not known.
    ///
    /// In most environments this indicates that one should use whatever the
    /// minimum nonzero amount is.
    #[default]
    Unknown,
    /// The amount of bytes required is exactly known.
    Known(NonZeroUsize),
}

impl Size {
    /// The minimum amount of **nonzero** bytes.
    pub const MIN: Size = Size::Known(NonZeroUsize::MIN);

    /// The maximum amount of **nonzero** bytes.
    pub const MAX: Size = Size::Known(NonZeroUsize::MAX);

    /// Create a new [`Size`].
    ///
    /// # Returns
    ///
    /// - `bytes == 0` becomes [`Size::Unknown`].
    /// - `bytes > 0` becomes [`Size::Known`].
    #[inline]
    pub const fn new(bytes: usize) -> Size {
        match NonZeroUsize::new(bytes) {
            Some(bytes) => Size::Known(bytes),
            None => Size::Unknown,
        }
    }

    /// Gets the known amount, if it is known.
    #[inline]
    pub const fn get(self) -> Option<NonZeroUsize> {
        match self {
            Size::Unknown => None,
            Size::Known(bytes) => Some(bytes),
        }
    }

    /// Gets the known amount, or return the provided default.
    #[inline]
    #[must_use]
    pub const fn get_or(self, default: NonZeroUsize) -> NonZeroUsize {
        match self {
            Size::Unknown => default,
            Size::Known(bytes) => bytes,
        }
    }

    /// Gets the known amount or returns a default of `1`.
    #[inline]
    #[must_use]
    pub const fn get_or_one(self) -> NonZeroUsize {
        self.get_or(NonZeroUsize::MIN)
    }

    /// Gets the known amount, or return a default computed from
    /// a closure.
    #[inline]
    #[must_use]
    pub fn get_or_else<F>(self, f: F) -> NonZeroUsize
    where
        F: FnOnce() -> NonZeroUsize,
    {
        match self {
            Size::Unknown => f(),
            Size::Known(bytes) => bytes,
        }
    }

    /// Returns whether the amount needed is known.
    #[inline]
    #[must_use]
    pub const fn is_known(self) -> bool {
        matches!(self, Self::Known(..))
    }

    /// Returns whether the amount needed is unknown.
    #[inline]
    #[must_use]
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Maps the known needed amount by applying a closure to it.
    #[inline]
    pub fn map<F>(self, f: F) -> Size
    where
        F: FnOnce(NonZeroUsize) -> usize,
    {
        match self {
            Size::Unknown => Size::Unknown,
            Size::Known(bytes) => Size::new(f(bytes)),
        }
    }
}

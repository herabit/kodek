use core::num::NonZeroUsize;

/// Represents a **nonzero** amount of bytes
/// that is required to perform some action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[must_use]
pub enum Needed {
    /// The amount of bytes required is not known.
    ///
    /// # Buffers
    ///
    /// This signifies to any buffers that are too small that it should grow
    /// by whatever their default growth factor is.
    ///
    /// # Readers
    ///
    /// This signifies to a reader that it should read as much data as it can.
    #[default]
    Unknown,
    /// The amount of bytes required is exactly known.
    ///
    /// # Buffers
    ///
    /// This signifies to any buffers that are too small that they should grow
    /// by exactly this amount.
    ///
    /// # Readers
    ///
    /// This signified to a reader that it needs to read exactly this amount.
    Known(NonZeroUsize),
}

impl Needed {
    /// Create a new [`Needed`].
    ///
    /// # Returns
    ///
    /// - `bytes_needed == 0` becomes [`Needed::Unknown`].
    /// - `bytes_needed > 0` becomes [`Needed::Known`].
    #[inline]
    pub const fn new(bytes_needed: usize) -> Needed {
        match NonZeroUsize::new(bytes_needed) {
            Some(bytes_needed) => Needed::Known(bytes_needed),
            None => Needed::Unknown,
        }
    }

    /// Gets the known amount, if it is known.
    #[inline]
    pub const fn get(self) -> Option<NonZeroUsize> {
        match self {
            Needed::Unknown => None,
            Needed::Known(bytes_needed) => Some(bytes_needed),
        }
    }

    /// Gets the known amount, or return the provided default.
    #[inline]
    #[must_use]
    pub const fn get_or(self, default: NonZeroUsize) -> NonZeroUsize {
        match self {
            Needed::Unknown => default,
            Needed::Known(bytes_needed) => bytes_needed,
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
            Needed::Unknown => f(),
            Needed::Known(bytes_needed) => bytes_needed,
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
    pub fn map<F>(self, f: F) -> Needed
    where
        F: FnOnce(NonZeroUsize) -> usize,
    {
        match self {
            Needed::Unknown => Needed::Unknown,
            Needed::Known(bytes_needed) => Needed::new(f(bytes_needed)),
        }
    }
}

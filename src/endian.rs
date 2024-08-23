use core::fmt;

pub const LE: LittleEndian = LittleEndian;
pub const BE: BigEndian = BigEndian;
pub const NE: NativeEndian = NativeEndian;

/// The endianness, or byte order of a stream of data.
///
/// # Default Value
///
/// Calling [`Default::default`] for [`Endian`] returns [`Endian::NATIVE`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub enum Endian {
    /// Little endian byte order.
    Little,
    /// Big endian byte order.
    Big,
}

impl Endian {
    /// This machine's native byte order.
    pub const NATIVE: Self = if cfg!(target_endian = "little") {
        Self::Little
    } else {
        Self::Big
    };

    /// Returns whether this is little endian.
    #[inline]
    #[must_use]
    pub const fn is_little(self) -> bool {
        matches!(self, Self::Little)
    }

    /// Returns whether this is big endian.
    #[inline]
    #[must_use]
    pub const fn is_big(self) -> bool {
        matches!(self, Self::Big)
    }

    /// Returns whether this is the native byte order.
    #[inline]
    #[must_use]
    pub const fn is_native(self) -> bool {
        matches!(self, Self::NATIVE)
    }

    /// Get the inverse endianness.
    ///
    /// - [`Endian::Little`] becomes [`Endian::Big`].
    /// - [`Endian::Big`] becomes [`Endian::Little`].
    #[inline]
    #[must_use]
    pub const fn to_inverse(self) -> Endian {
        match self {
            Endian::Little => Endian::Big,
            Endian::Big => Endian::Little,
        }
    }
}

impl Default for Endian {
    #[inline]
    fn default() -> Self {
        Endian::NATIVE
    }
}

impl core::ops::Not for Endian {
    type Output = Endian;

    /// Get the inverse endianness.
    ///
    /// See [`Endian::to_inverse`] for details.
    #[inline]
    fn not(self) -> Self::Output {
        self.to_inverse()
    }
}

impl core::ops::Not for &Endian {
    type Output = Endian;

    #[inline]
    fn not(self) -> Self::Output {
        self.to_inverse()
    }
}

impl ByteOrder for Endian {
    #[inline]
    fn endian(&self) -> Endian {
        *self
    }
}

/// Type for little endian byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LittleEndian;

impl ByteOrder for LittleEndian {
    #[inline]
    fn endian(&self) -> Endian {
        Endian::Little
    }
}

/// Type for big endian byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BigEndian;

impl ByteOrder for BigEndian {
    #[inline]
    fn endian(&self) -> Endian {
        Endian::Big
    }
}

/// Type for native endian byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NativeEndian;

impl ByteOrder for NativeEndian {
    #[inline]
    fn endian(&self) -> Endian {
        Endian::NATIVE
    }
}

/// Trait for types that represent a byte order.
pub trait ByteOrder: 'static + Copy + fmt::Debug {
    fn endian(&self) -> Endian;
}

impl ByteOrder for () {
    #[inline]
    fn endian(&self) -> Endian {
        Endian::NATIVE
    }
}

use core::{convert::Infallible, fmt, mem, num::NonZero};

use crate::{endian::Endian, sealed::Sealed};

/// Trait for Rust's sized primitive types, including the unit type ([`()`]).
pub trait Primitive: 'static + Copy + Sealed + fmt::Debug {
    type Bytes: 'static + Copy + Sealed + AsRef<[u8]> + AsMut<[u8]> + fmt::Debug + Default;
    type Error: 'static + Copy + fmt::Debug + fmt::Display;

    const BYTE_LEN: usize = mem::size_of::<Self>();

    #[must_use]
    fn try_from_bytes(bytes: Self::Bytes, endian: Endian) -> Result<Self, Self::Error>;

    #[must_use]
    fn to_bytes(self, endian: Endian) -> Self::Bytes;

    #[must_use]
    #[inline]
    fn from_bytes(bytes: Self::Bytes, endian: Endian) -> Self {
        Primitive::try_from_bytes(bytes, endian).unwrap()
    }
}

impl Primitive for () {
    type Bytes = [u8; 0];
    type Error = Infallible;

    #[inline]
    fn try_from_bytes(_: Self::Bytes, _: Endian) -> Result<Self, Self::Error> {
        Ok(())
    }

    #[inline]
    fn to_bytes(self, _: Endian) -> Self::Bytes {
        []
    }

    #[inline]
    fn from_bytes(_: Self::Bytes, _: Endian) -> Self {
        ()
    }
}

impl Primitive for bool {
    type Bytes = [u8; 1];
    type Error = BoolError;

    #[inline]
    fn try_from_bytes([b]: Self::Bytes, _: Endian) -> Result<Self, Self::Error> {
        match b {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BoolError(b)),
        }
    }

    #[inline]
    fn to_bytes(self, _: Endian) -> Self::Bytes {
        [self as u8]
    }
}

impl Primitive for char {
    type Bytes = [u8; 4];
    type Error = CharError;

    #[inline]
    fn try_from_bytes(bytes: Self::Bytes, endian: Endian) -> Result<Self, Self::Error> {
        let ch = u32::from_bytes(bytes, endian);

        match char::from_u32(ch) {
            Some(ch) => Ok(ch),
            None => Err(CharError(ch)),
        }
    }

    #[inline]
    fn to_bytes(self, endian: Endian) -> Self::Bytes {
        (self as u32).to_bytes(endian)
    }
}

/// Trait for primitives that are disallowed to be zero.
pub unsafe trait NonZeroPrimitive: Primitive {
    /// The zeroable variant of this type.
    type Zeroable: ZeroablePrimitive<
        NonZero = Self,
        Bytes = <Self as Primitive>::Bytes,
        Error = Infallible,
    >;

    #[must_use]
    fn from_zeroable(value: Self::Zeroable) -> Result<Self, NonZeroError<Self::Zeroable>>;

    #[must_use]
    fn to_zeroable(self) -> Self::Zeroable;
}

/// Trait for primitives that are allowed to be zero but have a nonzero variant.
pub unsafe trait ZeroablePrimitive: Primitive {
    /// The nonzero variant of this type.
    type NonZero: NonZeroPrimitive<
        Zeroable = Self,
        Bytes = <Self as Primitive>::Bytes,
        Error = NonZeroError<Self>,
    >;

    #[must_use]
    #[inline]
    fn to_nonzero(self) -> Result<Self::NonZero, NonZeroError<Self>> {
        Self::NonZero::from_zeroable(self)
    }

    #[must_use]
    #[inline]
    fn from_nonzero(value: Self::NonZero) -> Self {
        value.to_zeroable()
    }
}

macro_rules! prim {
    (@int $($ty:ty),* $(,)?) => {
        prim!($($ty),*);

        $(
            impl Primitive for NonZero<$ty> {
                type Bytes = [u8; mem::size_of::<$ty>()];
                type Error = NonZeroError<$ty>;

                #[inline]
                fn try_from_bytes(bytes: Self::Bytes, endian: Endian) -> Result<Self, Self::Error> {
                    let zeroable = <$ty>::from_bytes(bytes, endian);

                    Self::from_zeroable(zeroable)
                }

                #[inline]
                fn to_bytes(self, endian: Endian) -> Self::Bytes {
                    self.to_zeroable().to_bytes(endian)
                }
            }

            unsafe impl NonZeroPrimitive for NonZero<$ty> {
                type Zeroable = $ty;

                #[inline]
                #[must_use]
                fn from_zeroable(value: $ty) -> Result<Self, NonZeroError<$ty>> {
                    match Self::new(value) {
                        Some(value) => Ok(value),
                        None => Err(NonZeroError(value)),
                    }
                }

                #[inline]
                #[must_use]
                fn to_zeroable(self) -> $ty {
                    self.get()
                }
            }

            unsafe impl ZeroablePrimitive for $ty {
                type NonZero = NonZero<$ty>;
            }
        )*
    };

    ($($ty:ty),* $(,)?) => {
        $(
            impl Primitive for $ty {
                type Bytes = [u8; mem::size_of::<$ty>()];
                type Error = Infallible;

                #[inline]
                fn from_bytes(bytes: Self::Bytes, endian: Endian) -> Self {
                    match endian {
                        Endian::Little => Self::from_le_bytes(bytes),
                        Endian::Big => Self::from_be_bytes(bytes)
                    }
                }

                #[inline]
                fn to_bytes(self, endian: Endian) -> Self::Bytes {
                    match endian {
                        Endian::Little => self.to_le_bytes(),
                        Endian::Big => self.to_be_bytes(),
                    }
                }

                #[inline]
                fn try_from_bytes(bytes: Self::Bytes, endian: Endian) -> Result<Self, Self::Error> {
                    Ok(Self::from_bytes(bytes, endian))
                }
            }
        )*
    };
}

prim!(@int u8, u16, u32, u64, u128, usize);
prim!(@int i8, i16, i32, i64, i128, isize);
prim!(f32, f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharError(pub u32);

impl fmt::Display for CharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unicode code point out of range")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CharError {}

impl From<Infallible> for CharError {
    #[inline]
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoolError(pub u8);

impl fmt::Display for BoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid byte for a bool")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BoolError {}

impl From<Infallible> for BoolError {
    #[inline]
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonZeroError<T: ZeroablePrimitive>(pub T);

impl<T: ZeroablePrimitive> fmt::Display for NonZeroError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("primitive cannot be zero")
    }
}

#[cfg(feature = "std")]
impl<T: ZeroablePrimitive> std::error::Error for NonZeroError<T> {}

impl<T: ZeroablePrimitive> From<Infallible> for NonZeroError<T> {
    #[inline]
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

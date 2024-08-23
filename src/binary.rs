use core::{char::CharTryFromError, fmt};

use crate::decoder::{Decoder, Error as DError, Result as DResult};
use crate::endian::{ByteOrder, Endian, NativeEndian};
use crate::Size;

/// A binary [`Decoder`] that is capable of reading a [`prim@bool`]
/// in a specified byte order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bool<B: ByteOrder = NativeEndian> {
    /// The byte order for this decoder.
    pub byte_order: B,
}

impl<B: ByteOrder> Bool<B> {
    /// Create a new decoder for a [`prim@bool`].
    #[inline]
    #[must_use]
    pub const fn new(byte_order: B) -> Bool<B> {
        Bool { byte_order }
    }
}

impl<B: ByteOrder + Default> Default for Bool<B> {
    #[inline]
    fn default() -> Self {
        Bool::new(B::default())
    }
}

impl<B: ByteOrder> Decoder for Bool<B> {
    type Item<'src> = bool;
    type Error = BoolError;

    #[inline]
    fn hint(&self) -> Size {
        Size::new(1)
    }

    #[inline]
    fn decode<'s>(&mut self, src: &mut &'s [u8]) -> DResult<'s, Self> {
        let mut _src = *src;

        U8::new(self.byte_order)
            .decode(&mut _src)
            .map_err(DError::from_infallible)
            .and_then(|bits| match bits {
                0 => {
                    *src = _src;
                    Ok(false)
                }
                1 => {
                    *src = _src;
                    Ok(true)
                }
                _ => Err(DError::Fatal {
                    error: BoolError(()),
                }),
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoolError(());

impl fmt::Display for BoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid bits for a bool")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BoolError {}

/// A binary [`Decoder`] that is capable of reading a [`prim@char`]
/// in a specified byte order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Char<B: ByteOrder = NativeEndian> {
    /// The byte order for this decoder.
    pub byte_order: B,
}

impl<B: ByteOrder> Char<B> {
    /// Create a new decoder for a [`prim@char`].
    #[inline]
    #[must_use]
    pub const fn new(byte_order: B) -> Char<B> {
        Char { byte_order }
    }
}

impl<B: ByteOrder + Default> Default for Char<B> {
    #[inline]
    fn default() -> Self {
        Char::new(B::default())
    }
}

impl<B: ByteOrder> Decoder for Char<B> {
    type Item<'src> = char;
    type Error = CharTryFromError;

    #[inline]
    fn hint(&self) -> Size {
        Size::new(4)
    }

    #[inline]
    fn decode<'s>(&mut self, src: &mut &'s [u8]) -> DResult<'s, Self> {
        let mut _src = *src;

        U32::new(self.byte_order)
            .decode(&mut _src)
            .map_err(DError::from_infallible)
            .and_then(|bits| match char::try_from(bits) {
                Ok(ch) => {
                    *src = _src;
                    Ok(ch)
                }
                Err(error) => Err(DError::Fatal { error }),
            })
    }
}

macro_rules! define {
    ($(
        $(#[$attr:meta])*
        $vis:vis struct $name:ident<$ty:ident> {}
    )*) => {
        $(
            #[doc = ::core::concat!(
                "A binary [`Decoder`] that is capable of reading a [`prim@",
                ::core::stringify!($ty),
                "`] in a specified byte order.",
            )]
            $(#[$attr])*
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            $vis struct $name<B: ByteOrder = NativeEndian> {
                /// The byte order for this decoder.
                pub byte_order: B,
            }

            impl<B: ByteOrder> $name<B> {
                const SIZE: usize = ::core::mem::size_of::<::core::primitive::$ty>();

                #[doc = ::core::concat!(
                    "Create a new binary decoder for a [`prim@",
                    ::core::stringify!($ty),
                    "`]."
                )]
                #[inline]
                #[must_use]
                pub const fn new(byte_order: B) -> $name<B> {
                    $name {
                        byte_order,
                    }
                }

            }

            impl<B: ByteOrder + Default> Default for $name<B> {
                #[inline]
                fn default() -> Self {
                    Self::new(B::default())
                }
            }

            impl<B: ByteOrder> Decoder for $name<B> {
                type Item<'src> = ::core::primitive::$ty;
                type Error = ::core::convert::Infallible;

                #[inline]
                fn hint(&self) -> Size {
                    Size::new(Self::SIZE)
                }

                #[inline]
                fn decode<'s>(&mut self, src: &mut &'s [u8]) -> DResult<'s, Self> {
                    let Some((bytes, rest)) = src.split_at_checked(Self::SIZE) else {
                        return Err(DError::Incomplete { needed: Size::new(Self::SIZE - src.len()) });
                    };

                    let bytes: [u8; $name::<()>::SIZE] = bytes.try_into().unwrap();
                    let bits = match self.byte_order.endian() {
                        Endian::Little => ::core::primitive::$ty::from_le_bytes(bytes),
                        Endian::Big => ::core::primitive::$ty::from_be_bytes(bytes),
                    };

                    *src = rest;

                    Ok(bits)
                }
            }
        )*
    };
}

define! {
    pub struct U8<u8> {}
    pub struct U16<u16> {}
    pub struct U32<u32> {}
    pub struct U64<u64> {}
    pub struct U128<u128> {}
    pub struct Usize<usize> {}

    pub struct I8<i8> {}
    pub struct I16<i16> {}
    pub struct I32<i32> {}
    pub struct I64<i64> {}
    pub struct I128<i128> {}
    pub struct Isize<isize> {}

    pub struct F32<f32> {}
    pub struct F64<f64> {}
}

use core::{convert::Infallible, fmt};

use crate::{endian::Endian, primitive::Primitive};

/// A falliable equivalent to [`::bytes::Buf`].
pub trait Buffer {
    type Error: fmt::Debug;

    fn chunk(&self) -> &[u8];
    fn remaining(&self) -> usize;

    fn try_advance(&mut self, cnt: usize) -> Result<(), Self::Error>;
    fn try_copy_to_slice(&mut self, slice: &mut [u8]) -> Result<(), Self::Error>;

    #[inline]
    fn read_with<T: ReadBuffer<Ctx>, Ctx>(&mut self, ctx: Ctx) -> Result<T, T::Error<Self>> {
        T::try_read_buffer(self, ctx)
    }

    #[inline]
    fn read<T: ReadBuffer<()>>(&mut self) -> Result<T, T::Error<Self>> {
        T::try_read_buffer(self, ())
    }
}

impl Buffer for &[u8] {
    type Error = &'static str;

    #[inline]
    fn chunk(&self) -> &[u8] {
        &**self
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn try_advance(&mut self, cnt: usize) -> Result<(), Self::Error> {
        match self.split_at_checked(cnt) {
            Some((_, rest)) => {
                *self = rest;
                Ok(())
            }
            None => Err("failed to advance slice"),
        }
    }

    #[inline]
    fn try_copy_to_slice(&mut self, slice: &mut [u8]) -> Result<(), Self::Error> {
        match self.split_at_checked(slice.len()) {
            Some((bytes, rest)) => {
                slice.copy_from_slice(bytes);
                *self = rest;

                Ok(())
            }
            None => Err("failed to advance slice"),
        }
    }
}

pub trait ReadBuffer<Ctx = ()>: Sized {
    type Error<B: Buffer + ?Sized>: ReadError<Ctx> + FromBufferError<B>;

    fn try_read_buffer<B: Buffer + ?Sized>(
        buffer: &mut B,
        context: Ctx,
    ) -> Result<Self, Self::Error<B>>;

    #[inline]
    #[must_use]
    fn read_buffer<B: Buffer + ?Sized>(buffer: &mut B, context: Ctx) -> Self {
        Self::try_read_buffer(buffer, context).unwrap()
    }
}

impl<P: Primitive> ReadBuffer<Endian> for P {
    type Error<B: Buffer + ?Sized> = ReadPrimitiveError<P, B>;

    #[inline]
    fn try_read_buffer<B: Buffer + ?Sized>(
        buffer: &mut B,
        endian: Endian,
    ) -> Result<Self, Self::Error<B>> {
        let mut bytes = P::Bytes::default();

        buffer
            .try_copy_to_slice(bytes.as_mut())
            .map_err(FromBufferError::from_buffer_error)?;

        P::try_from_bytes(bytes, endian)
            .map_err(|error| ReadPrimitiveError::Primitive { endian, error })
    }
}

impl<P: Primitive> ReadBuffer<()> for P {
    type Error<B: Buffer + ?Sized> = ReadPrimitiveError<P, B>;

    #[inline]
    fn try_read_buffer<B: Buffer + ?Sized>(buffer: &mut B, _: ()) -> Result<Self, Self::Error<B>> {
        Self::try_read_buffer(buffer, Endian::default())
    }
}

pub trait ReadError<Ctx = ()>: fmt::Debug + Sized {
    #[inline]
    fn context(&self) -> Option<&Ctx> {
        None
    }

    #[inline]
    fn with_context(self, _: Ctx) -> Self {
        self
    }
}

impl<Ctx> ReadError<Ctx> for () {}
impl<Ctx> ReadError<Ctx> for Infallible {}

pub trait FromBufferError<B: Buffer + ?Sized>: Sized {
    #[must_use]
    fn from_buffer_error(error: B::Error) -> Self;
}

impl<B: Buffer + ?Sized> FromBufferError<B> for () {
    #[inline]
    fn from_buffer_error(_: <B as Buffer>::Error) -> Self {
        ()
    }
}

impl<B: Buffer + ?Sized> FromBufferError<B> for Infallible {
    #[inline]
    #[track_caller]
    fn from_buffer_error(error: <B as Buffer>::Error) -> Self {
        panic!("error: {error:?}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReadPrimitiveError<P, B>
where
    P: Primitive,
    B: Buffer + ?Sized,
{
    Primitive { endian: Endian, error: P::Error },
    Buffer { endian: Endian, error: B::Error },
}

impl<P, B> ReadPrimitiveError<P, B>
where
    P: Primitive,
    B: Buffer + ?Sized,
{
    #[inline]
    pub const fn endian(&self) -> Endian {
        match self {
            Self::Primitive { endian, .. } => *endian,
            Self::Buffer { endian, .. } => *endian,
        }
    }
}

impl<P, B> fmt::Display for ReadPrimitiveError<P, B>
where
    P: Primitive,
    P::Error: fmt::Display,
    B: Buffer + ?Sized,
    B::Error: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive { error, .. } => error.fmt(f),
            Self::Buffer { error, .. } => error.fmt(f),
        }
    }
}

impl<P, B> fmt::Debug for ReadPrimitiveError<P, B>
where
    P: Primitive,
    B: Buffer + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive { error, .. } => f.debug_tuple("Primitive").field(error).finish(),
            Self::Buffer { error, .. } => f.debug_tuple("Buffer").field(error).finish(),
        }
    }
}

impl<P, B> ReadError<Endian> for ReadPrimitiveError<P, B>
where
    P: Primitive,
    B: Buffer + ?Sized,
{
    #[inline]
    fn context(&self) -> Option<&Endian> {
        match self {
            Self::Primitive { endian, .. } => Some(endian),
            Self::Buffer { endian, .. } => Some(endian),
        }
    }

    #[inline]
    fn with_context(self, endian: Endian) -> Self {
        match self {
            Self::Primitive { error, .. } => Self::Primitive { endian, error },
            Self::Buffer { error, .. } => Self::Buffer { endian, error },
        }
    }
}

impl<P, B> ReadError<()> for ReadPrimitiveError<P, B>
where
    P: Primitive,
    B: Buffer + ?Sized,
{
    #[inline]
    fn with_context(self, _: ()) -> Self {
        let endian = Endian::default();

        self.with_context(endian)
    }
}

impl<P, B> FromBufferError<B> for ReadPrimitiveError<P, B>
where
    P: Primitive,
    B: Buffer + ?Sized,
{
    #[inline]
    fn from_buffer_error(error: <B as Buffer>::Error) -> Self {
        Self::Buffer {
            endian: Endian::default(),
            error,
        }
    }
}

#[test]
fn test_read() {
    extern crate std;

    use std::vec::Vec;

    let mut bytes = Vec::new();

    bytes.extend(f32::NAN.to_bits().to_be_bytes());
    bytes.extend((char::REPLACEMENT_CHARACTER as u32).to_le_bytes());

    let mut buffer = bytes.as_slice();

    assert!(buffer.read::<f32>().unwrap().is_nan());
    assert!(buffer.read_with::<char, _>(Endian::Little).unwrap() == char::REPLACEMENT_CHARACTER);
}

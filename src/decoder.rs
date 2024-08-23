use core::{convert::Infallible, fmt, write};

use crate::Size;

/// Various binary decoders.
pub mod binary;

#[allow(type_alias_bounds)]
pub type Result<'s, D: Decoder> = ::core::result::Result<D::Item<'s>, Error<D::Error>>;

/// Trait for decoders.
pub trait Decoder {
    type Item<'src>;
    type Error: fmt::Display + fmt::Debug;

    /// Get an estimate for the amount of bytes required to read the next frame.
    ///
    /// This should depend on the overall state of the decoder, not previous attempts
    /// to parse the next frame.
    #[inline]
    fn hint(&self) -> Size {
        Size::Unknown
    }

    /// Try to decode a single frame from a byte stream.
    fn decode<'s>(&mut self, src: &mut &'s [u8]) -> Result<'s, Self>;

    /// Try to decode the last frame from a byte stream.
    #[inline]
    fn decode_eof<'s>(&mut self, src: &mut &'s [u8]) -> Result<'s, Self> {
        match self.decode(src) {
            Ok(item) => Ok(item),
            Err(Error::Fatal { error }) => Err(Error::Fatal { error }),
            Err(_) if src.is_empty() => Err(Error::Eof),
            Err(_) => Err(Error::DataRemains),
        }
    }
}

/// Type for errors that may occur while decoding a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error<E> {
    /// We've reached the end of a stream.
    ///
    /// There are no more frames to decode.
    Eof,
    /// Bytes remain in the stream despite being at the end of it.
    DataRemains,
    /// The data read so far looks to be valid but
    /// the input was incomplete.
    ///
    /// Do not advance the source buffer when returning this.
    Incomplete {
        /// The minimum amount of bytes required for
        /// reading the next frame.
        needed: Size,
    },
    /// A fatal error has occurred while reading the
    /// current frame.
    ///
    /// This indicates that the stream is corrupt and should
    /// be terminated.
    Fatal {
        /// The error.
        error: E,
    },
}

impl<E> Error<E> {
    #[inline]
    #[must_use]
    pub fn map<T, F: FnOnce(E) -> T>(self, f: F) -> Error<T> {
        match self {
            Error::Eof => Error::Eof,
            Error::DataRemains => Error::DataRemains,
            Error::Incomplete { needed } => Error::Incomplete { needed },
            Error::Fatal { error } => Error::Fatal { error: f(error) },
        }
    }

    #[inline]
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::Eof => "reached end of stream",
            Self::DataRemains => "data remains in stream",
            Self::Incomplete { .. } => "incomplete frame",
            Self::Fatal { .. } => "fatal error occurred",
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_infallible(error: Error<Infallible>) -> Error<E> {
        match error {
            Error::Eof => Error::Eof,
            Error::DataRemains => Error::DataRemains,
            Error::Incomplete { needed } => Error::Incomplete { needed },
        }
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())?;

        match self {
            Self::Eof | Self::DataRemains => Ok(()),
            Self::Incomplete {
                needed: Size::Unknown,
            } => f.write_str(": requires more data"),
            Self::Incomplete {
                needed: Size::Known(n),
            } => write!(f, ": requires at least {n} bytes"),
            Self::Fatal { error } => write!(f, ": {error}"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Display + fmt::Debug> std::error::Error for Error<E> {}

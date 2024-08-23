#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod size;

/// Types and traits relating to byte ordering.
pub mod endian;

/// Types and traits relating to decoders.
pub mod decoder;

/// Encoders and decoders for binary data.
pub mod binary;

#[doc(inline)]
pub use decoder::Decoder;

#[doc(inline)]
pub use size::Size;

#[doc(inline)]
pub use endian::Endian;

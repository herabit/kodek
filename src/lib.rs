#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod endian;
pub use endian::Endian;

mod size;
pub use size::Size;

pub mod decoder;

// mod zeroable;
// pub use zeroable::*;

// mod bytes;
// pub use bytes::*;

// mod array;

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod endian;
pub use endian::Endian;

mod needed;
pub use needed::Needed;

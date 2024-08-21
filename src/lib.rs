#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod read_buf;

pub mod buffer;
pub mod buffer_mut;

pub mod endian;
pub mod primitive;

mod sealed;

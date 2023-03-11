#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod mmap;
pub use mmap::*;

mod error;
pub use error::*;



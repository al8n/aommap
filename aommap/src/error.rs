#[cfg(feature = "std")]
mod rust;
#[cfg(feature = "std")]
pub use rust::*;

#[cfg(not(feature = "std"))]
mod wasm;
#[cfg(not(feature = "std"))]
pub use wasm::*;

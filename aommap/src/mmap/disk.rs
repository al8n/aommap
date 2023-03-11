#![allow(dead_code)]

use core::sync::atomic::AtomicUsize;

mod immutable;
pub use immutable::*;
mod mutable;
pub use mutable::*;


#[derive(Debug)]
struct Inner {
  file: std::fs::File,
  cursor: AtomicUsize,
}

impl Inner {
  #[inline]
  fn new(file: std::fs::File, cursor: usize) -> Self {
    Self { file, cursor: AtomicUsize::new(cursor) }
  }
}
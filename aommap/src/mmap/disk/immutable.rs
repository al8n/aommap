use crate::error::{Error, Result};
use memmapix::Mmap as Map;
use std::fs::OpenOptions;

use super::Inner;

/// Read-only memory map.
#[derive(Debug)]
pub struct Mmap {
  pub(super) inner: Inner,
  pub(super) map: Map,
  pub(super) len: usize,
}

impl Mmap {
  /// Open a existing file and mmap it.
  #[inline]
  pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.read(true);
    opts.open(path).map_err(From::from).and_then(|file| {
      let meta = file.metadata().map_err(Error::IO)?;
      let len = meta.len() as usize;
      // Safety: we just open the file, this file is valid.
      unsafe { Map::map(&file) }
        .map(|map| Self {
          inner: Inner::new(file, len),
          map,
          len,
        })
        .map_err(Error::Mmap)
    })
  }
}

impl AsRef<[u8]> for Mmap {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    &self.map[..self.len]
  }
}

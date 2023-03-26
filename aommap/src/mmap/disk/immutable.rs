use crate::error::{Error, Result};
use memmapix::Mmap as Map;
use std::fs::OpenOptions;

use super::Inner;

#[derive(Debug)]
struct MmapInner {
  pub(super) inner: Inner,
  pub(super) map: Map,
  pub(super) len: usize,
}

/// Read-only memory map, thread-safe.
///
/// **Note:** when using Mmap, users do not need to wrap it with a `Arc` to share it between threads.
#[derive(Debug, Clone)]
pub struct Mmap(std::sync::Arc<MmapInner>);

impl Mmap {
  pub(super) fn new(inner: Inner, map: Map, len: usize) -> Self {
    Self(std::sync::Arc::new(MmapInner { inner, map, len }))
  }

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
        .map(|map| Self::new(Inner::new(file, len), map, len))
        .map_err(Error::Mmap)
    })
  }
}

impl AsRef<[u8]> for Mmap {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    &self.0.map[..self.0.len]
  }
}

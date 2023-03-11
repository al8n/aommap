use crate::error::{Error, Result};
use core::sync::atomic::Ordering;
use memmapix::MmapMut as Map;
use std::fs::OpenOptions;

use super::Inner;

#[derive(Debug)]
pub struct MmapMut {
  inner: Inner,
  map: Map,
}

impl MmapMut {
  /// Create a new file if the file is not exist and mmap it.
  #[inline]
  pub fn create<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.create(true).read(true).write(true);
    Self::open_in(path.as_ref(), opts)
  }

  /// Create a new file and mmap it.
  #[inline]
  pub fn create_new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.create_new(true).read(true).write(true);
    Self::open_in(path.as_ref(), opts)
  }

  /// Open a existing file and mmap it.
  #[inline]
  pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.create(false).read(true).write(true);
    Self::open_in(path.as_ref(), opts)
  }

  /// Open a file with given options and mmap it.
  #[inline]
  pub fn open_with_options<P: AsRef<std::path::Path>>(
    path: P,
    open_opts: OpenOptions,
  ) -> Result<Self> {
    Self::open_in(path.as_ref(), open_opts)
  }

  #[inline]
  fn open_in(path: &std::path::Path, open_opts: OpenOptions) -> Result<Self> {
    open_opts.open(path).map_err(From::from).and_then(|file| {
      let meta = file.metadata().map_err(Error::IO)?;
      let len = meta.len() as usize;
      // Safety: we just open the file, this file is valid.
      unsafe { Map::map_mut(&file) }
        .map(|map| Self {
          inner: Inner::new(file, len),
          map,
        })
        .map_err(Error::Mmap)
    })
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.inner.cursor.load(Ordering::Relaxed)
  }

  /// This method is concurrent-safe, will write all of the data in the buf.
  #[inline]
  pub fn write(&self, buf: &[u8]) -> Result<()> {
    let cursor = self.inner.cursor.load(Ordering::SeqCst);
    let len = self.map.len();
    let buf_len = buf.len();
    let remaining = len - cursor;
    if buf_len >= remaining {
      return Err(Error::BufTooLarge);
    }
    self.inner.cursor.fetch_add(buf_len, Ordering::SeqCst);

    // Safety: we have a cursor to make sure there is no data race for the same range
    let slice = unsafe { core::slice::from_raw_parts_mut(self.map.as_ptr() as *mut u8, len) };
    slice[cursor..cursor + buf.len()].copy_from_slice(buf);
    Ok(())
  }

  #[inline]
  pub fn flush(&self) -> Result<()> {
    self.map.flush().map_err(Error::Flush)
  }

  #[inline]
  pub fn flush_async(&self) -> Result<()> {
    self.map.flush_async().map_err(Error::Flush)
  }

  #[inline]
  pub fn freeze(self) -> Result<super::immutable::Mmap> {
    let len = self.inner.cursor.load(Ordering::SeqCst);
    self
      .map
      .make_read_only()
      .map(|map| super::immutable::Mmap {
        inner: self.inner,
        map,
        len,
      })
      .map_err(Error::Mmap)
  }
}

impl AsRef<[u8]> for MmapMut {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    &self.map[..self.len()]
  }
}

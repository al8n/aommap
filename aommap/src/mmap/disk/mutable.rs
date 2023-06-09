use crate::error::{Error, Result};
use core::{ops::RangeBounds, sync::atomic::Ordering};
use memmapix::{MmapMut as Map, MmapOptions};
use std::fs::OpenOptions;

use super::Inner;

#[derive(Debug)]
pub struct MmapMut {
  inner: Inner,
  map: Map,
  cap: usize,
}

impl MmapMut {
  /// Create a file if the file is not exist, otherwise truncate file. Then, mmap it.
  #[inline]
  pub fn create<P: AsRef<std::path::Path>>(path: P, max_size: usize) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.create(true).read(true).write(true).truncate(true);
    Self::open_in(path.as_ref(), opts, max_size)
  }

  /// Create a new file and mmap it.
  #[inline]
  pub fn create_new<P: AsRef<std::path::Path>>(path: P, max_size: usize) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.create_new(true).read(true).write(true);
    Self::open_in(path.as_ref(), opts, max_size)
  }

  /// Open a existing file and mmap it.
  #[inline]
  pub fn open<P: AsRef<std::path::Path>>(path: P, max_size: usize) -> Result<Self> {
    let mut opts = OpenOptions::new();
    opts.create(false).read(true).write(true);
    Self::open_in(path.as_ref(), opts, max_size)
  }

  /// Open a file with given options and mmap it.
  #[inline]
  pub fn open_with_options<P: AsRef<std::path::Path>>(
    path: P,
    open_opts: OpenOptions,
    max_size: usize,
  ) -> Result<Self> {
    Self::open_in(path.as_ref(), open_opts, max_size)
  }

  #[inline]
  fn open_in(path: &std::path::Path, open_opts: OpenOptions, max_size: usize) -> Result<Self> {
    open_opts.open(path).map_err(From::from).and_then(|file| {
      let meta = file.metadata().map_err(Error::IO)?;
      let len = meta.len() as usize;
      if len < max_size {
        file.set_len(max_size as u64).map_err(Error::IO)?;
      }
      let mut opts = MmapOptions::new();
      opts.len(max_size);
      // Safety: we just open the file, this file is valid.
      unsafe { opts.map_mut(&file) }
        .map(|map| Self {
          inner: Inner::new(file, len),
          map,
          cap: max_size,
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

  #[inline]
  pub fn slice(&self, range: impl RangeBounds<usize>) -> &[u8] {
    use core::ops::Bound;

    const EMPTY: &[u8] = &[];

    let len = self.len();

    let begin = match range.start_bound() {
      Bound::Included(&n) => n,
      Bound::Excluded(&n) => n + 1,
      Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
      Bound::Included(&n) => n.checked_add(1).expect("out of range"),
      Bound::Excluded(&n) => n,
      Bound::Unbounded => len,
    };

    assert!(
      begin <= end,
      "range start must not be greater than end: {:?} <= {:?}",
      begin,
      end,
    );
    assert!(
      end <= len,
      "range end out of bounds: {:?} <= {:?}",
      end,
      len,
    );

    if end == begin {
      return EMPTY;
    }

    &self.map[begin..end]
  }

  /// This method is concurrent-safe, will write all of the data in the buf, returns the offset of the data in the file.
  #[inline]
  pub fn write(&self, buf: &[u8]) -> Result<usize> {
    let buf_len = buf.len();
    let cursor = self.inner.cursor.fetch_add(buf_len, Ordering::SeqCst);
    let len = self.map.len();
    let remaining = len - cursor;
    if buf_len > remaining {
      return Err(Error::EOF);
    }

    // Safety: we have a cursor to make sure there is no data race for the same range
    let slice = unsafe { core::slice::from_raw_parts_mut(self.map.as_ptr() as *mut u8, len) };
    slice[cursor..cursor + buf.len()].copy_from_slice(buf);
    Ok(cursor)
  }

  /// This method is concurrent-safe, will return an offset and a mutable slice for you to write data.
  #[inline]
  pub fn bytes_mut(&self, buf_len: usize) -> Result<(usize, &mut [u8])> {
    let cursor = self.inner.cursor.fetch_add(buf_len, Ordering::SeqCst);
    let len = self.map.len();
    let remaining = len - cursor;
    if buf_len > remaining {
      return Err(Error::EOF);
    }

    // Safety: we have a cursor to make sure there is no data race for the same range
    let slice = unsafe { core::slice::from_raw_parts_mut(self.map.as_ptr() as *mut u8, len) };
    Ok((cursor, &mut slice[cursor..cursor + buf_len]))
  }

  /// Returns the underlying mutable slice of the mmap.
  ///
  /// # Safety
  /// This method is not thread-safe.
  #[inline]
  pub unsafe fn underlying_slice_mut(&self) -> &mut [u8] {
    core::slice::from_raw_parts_mut(self.map.as_ptr() as *mut u8, self.cap)
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
      .map(|map| super::immutable::Mmap::new(self.inner, map, len))
      .map_err(Error::Mmap)
  }
}

impl AsRef<[u8]> for MmapMut {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    &self.map[..self.len()]
  }
}

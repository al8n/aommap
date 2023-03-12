use crate::error::{Error, Result};
use bytes::{BufMut, BytesMut};
use core::{ops::RangeBounds, sync::atomic::AtomicUsize};

pub struct MmapMut {
  ptr: *mut BytesMut,
  cursor: AtomicUsize,
  cap: usize,
}

impl MmapMut {
  pub fn new(size: usize) -> Self {
    Self {
      ptr: Box::into_raw(Box::new(BytesMut::with_capacity(size))),
      cursor: AtomicUsize::new(0),
      cap: size,
    }
  }

  #[inline]
  pub fn slice(&self, range: impl RangeBounds<usize>) -> &[u8] {
    use core::ops::Bound;

    const EMPTY: &[u8] = &[];

    let len = self.cursor.load(core::sync::atomic::Ordering::SeqCst);

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

    let bytes = unsafe { &*self.ptr };
    &bytes[begin..end]
  }

  /// This method is concurrent-safe, will write all of the data in the buf, returns the offset of the data in the file.
  #[inline]
  pub fn write(&self, buf: &[u8]) -> Result<usize> {
    let buf_len = buf.len();
    let cursor = self.cursor.fetch_add(buf_len, core::sync::atomic::Ordering::SeqCst);
    let remaining = self.cap - cursor;
    if buf.len() > remaining {
      return Err(Error::EOF);
    }

    let bytes = unsafe { &mut *self.ptr };
    bytes.put_slice(buf);
    Ok(cursor)
  }

  /// This method is concurrent-safe, will return a mutable slice for you to write data.
  #[inline]
  pub fn writable_slice(&self, buf_len: usize) -> Result<&mut BytesMut> {
    let cursor = self.cursor.fetch_add(buf_len, core::sync::atomic::Ordering::SeqCst); 
    let remaining = self.cap - cursor;
    if buf_len > remaining {
      return Err(Error::EOF);
    }

    let bytes = unsafe { &mut *self.ptr };
    Ok(bytes)
  }
}

impl Drop for MmapMut {
  fn drop(&mut self) {
    unsafe { Box::from_raw(self.ptr) };
  }
}

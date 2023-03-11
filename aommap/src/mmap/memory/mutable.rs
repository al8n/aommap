use core::sync::atomic::AtomicUsize;
use crate::error::{Error, Result};
use bytes::{BytesMut, BufMut};


pub struct MmapMut {
  ptr: *mut BytesMut,
  cursor: AtomicUsize,
  cap: usize,
}

impl MmapMut {
  pub fn new(size: usize) -> Self {
    Self { ptr: Box::into_raw(Box::new(BytesMut::with_capacity(size))), cursor: AtomicUsize::new(0), cap: size }
  }

  /// This method is concurrent-safe, will write all of the data in the buf.
  #[inline]
  pub fn write(&self, buf: &[u8]) -> Result<()> {
    let remaining = self.cap - self.cursor.load(core::sync::atomic::Ordering::SeqCst);
    if buf.len() > remaining {
      return Err(Error::BufTooLarge);
    }
    self.cursor.fetch_add(buf.len(), core::sync::atomic::Ordering::SeqCst);

    let bytes = unsafe { &mut *self.ptr };
    bytes.put_slice(buf); 
    Ok(())
  }
}

impl Drop for MmapMut {
  fn drop(&mut self) {
    unsafe { Box::from_raw(self.ptr) };
  }
}
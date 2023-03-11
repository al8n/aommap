

#[derive(Debug, Clone)]
pub struct Mmap(bytes::Bytes);

impl Mmap {
  pub fn open(bytes: bytes::Bytes) -> Self {
    Self(bytes)
  }
}

impl AsRef<[u8]> for Mmap {
  #[inline]
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}
// use rustix::io::std::io::Error;
use std::io::Error as StdError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  LockMemory(StdError),
  UnlockMemory(StdError),
  MmapZero,
  Mmap(StdError),
  Unmmap(StdError),
  MmapAnonymous(StdError),
  Advice(StdError),
  Flock(StdError),
  Funlock(StdError),
  Flush(StdError),
  Mprotect(StdError),
  BufTooLarge,
  IO(StdError),
  EOF,
}

impl From<StdError> for Error {
  fn from(e: StdError) -> Self {
    Self::IO(e)
  }
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EOF => write!(f, "EOF: need to remmap"),
      Self::BufTooLarge => write!(f, "Buffer is too large"),
      Self::Flush(e) => write!(f, "Failed to flush: {}", e),
      Self::IO(e) => write!(f, "{}", e),
      Self::LockMemory(e) => write!(f, "Failed to mlock: {}", e),
      Self::UnlockMemory(e) => write!(f, "Failed to munlock: {}", e),
      Self::MmapZero => write!(f, "Failed to mmap: zero length"),
      Self::Mmap(e) => write!(f, "Failed to mmap: {}", e),
      Self::MmapAnonymous(e) => write!(f, "Failed to mmap anonymous: {}", e),
      Self::Mprotect(e) => write!(f, "Failed to mprotect: {}", e),
      Self::Unmmap(e) => write!(f, "Failed to munmap: {}", e),
      Self::Advice(e) => write!(f, "Failed to madvise: {}", e),
      Self::Flock(e) => write!(f, "Failed to flock: {}", e),
      Self::Funlock(e) => write!(f, "Failed to funlock: {}", e),
    }
  }
}

impl std::error::Error for Error {}

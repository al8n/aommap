
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  BufTooLarge,
}

impl core::fmt::Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::BufTooLarge => write!(f, "Buffer is too large"),
    }
  }
}
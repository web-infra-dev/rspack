use rkyv::rancor::{BoxedError, Source, Trace};

#[derive(Debug)]
pub enum Error {
  BoxedError(BoxedError),
  MessageError(&'static str),
  DynCheckBytesNotRegister,
  NoContext,
  UnsupportedField,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::BoxedError(error) => error.fmt(f),
      Self::MessageError(msg) => {
        write!(f, "{msg}")
      }
      Self::DynCheckBytesNotRegister => {
        write!(f, "cacheable_dyn check bytes not register")
      }
      Self::NoContext => {
        write!(f, "no context")
      }
      Self::UnsupportedField => {
        write!(f, "unsupported field")
      }
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::BoxedError(error) => error.source(),
      _ => None,
    }
  }
}

impl Trace for Error {
  fn trace<R>(self, trace: R) -> Self
  where
    R: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
  {
    Self::BoxedError(BoxedError::trace(BoxedError::new(self), trace))
  }
}

impl Source for Error {
  fn new<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
    Self::BoxedError(BoxedError::new(source))
  }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

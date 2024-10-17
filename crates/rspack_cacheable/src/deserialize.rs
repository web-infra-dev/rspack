use std::any::Any;

use rkyv::{
  access,
  api::{deserialize_using, high::HighValidator},
  bytecheck::CheckBytes,
  de::Pool,
  rancor::{BoxedError, Source, Strategy, Trace},
  Archive, Deserialize,
};

use crate::context::ContextGuard;

#[derive(Debug)]
pub enum DeserializeError {
  RkyvError(BoxedError),
  DynCheckBytesNotRegister,
  // A deserialize failed occurred
  DeserializeFailed(&'static str),
  NoContext,
  UnsupportedField,
}

impl std::fmt::Display for DeserializeError {
  fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //    write!(f, "{}", self.inner)?;
    Ok(())
  }
}

impl std::error::Error for DeserializeError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    //    self.inner.source()
    todo!()
  }
}

impl Trace for DeserializeError {
  fn trace<R>(self, _trace: R) -> Self
  where
    R: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
  {
    todo!()
    //    Self::RkyvError()
    //      inner: self.inner.trace(trace),
    //    }
  }
}

impl Source for DeserializeError {
  fn new<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
    Self::RkyvError(BoxedError::new(source))
  }
}

pub type Validator<'a> = HighValidator<'a, DeserializeError>;
pub type Deserializer = Strategy<Pool, DeserializeError>;

pub fn from_bytes<T, C: Any>(bytes: &[u8], context: &C) -> Result<T, DeserializeError>
where
  T: Archive,
  T::Archived: for<'a> CheckBytes<Validator<'a>> + Deserialize<T, Deserializer>,
{
  let guard = ContextGuard::new(context);
  let mut deserializer = Pool::default();
  guard.add_to_pooling(&mut deserializer)?;
  deserialize_using(
    access::<T::Archived, DeserializeError>(bytes)?,
    &mut deserializer,
  )
}

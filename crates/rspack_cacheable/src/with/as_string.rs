use rkyv::{
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use crate::{DeserializeError, SerializeError};

pub struct AsString;

pub trait AsStringConverter {
  fn to_string(&self) -> Result<String, SerializeError>;
  fn from_str(s: &str) -> Result<Self, DeserializeError>
  where
    Self: Sized;
}

pub struct AsStringResolver {
  inner: StringResolver,
  value: String,
}

impl<T> ArchiveWith<T> for AsString
where
  T: AsStringConverter,
{
  type Archived = ArchivedString;
  type Resolver = AsStringResolver;

  #[inline]
  fn resolve_with(_field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let AsStringResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<T, S> SerializeWith<T, S> for AsString
where
  T: AsStringConverter,
  S: Fallible<Error = SerializeError> + Writer + ?Sized,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, SerializeError> {
    let value = field.to_string()?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(AsStringResolver { value, inner })
  }
}

impl<T, D> DeserializeWith<ArchivedString, T, D> for AsString
where
  T: AsStringConverter,
  D: Fallible<Error = DeserializeError> + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<T, DeserializeError> {
    AsStringConverter::from_str(field.as_str())
  }
}

// for pathbuf
use std::path::PathBuf;
impl AsStringConverter for PathBuf {
  fn to_string(&self) -> Result<String, SerializeError> {
    Ok(self.to_string_lossy().to_string())
  }
  fn from_str(s: &str) -> Result<Self, DeserializeError>
  where
    Self: Sized,
  {
    Ok(PathBuf::from(s))
  }
}

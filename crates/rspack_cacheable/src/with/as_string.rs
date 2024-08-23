use rkyv::{
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

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
  unsafe fn resolve_with(
    _field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let AsStringResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, pos, inner, out);
  }
}

impl<T> SerializeWith<T, CacheableSerializer> for AsString
where
  T: AsStringConverter,
{
  #[inline]
  fn serialize_with(
    field: &T,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = field.to_string()?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(AsStringResolver { value, inner })
  }
}

impl<T> DeserializeWith<ArchivedString, T, CacheableDeserializer> for AsString
where
  T: AsStringConverter,
{
  #[inline]
  fn deserialize_with(
    field: &ArchivedString,
    _: &mut CacheableDeserializer,
  ) -> Result<T, DeserializeError> {
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

// for Box<str>
impl AsStringConverter for Box<str> {
  fn to_string(&self) -> Result<String, SerializeError> {
    Ok(str::to_string(self))
  }
  fn from_str(s: &str) -> Result<Self, DeserializeError>
  where
    Self: Sized,
  {
    Ok(s.into())
  }
}

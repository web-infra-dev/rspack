use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result};

pub struct AsString;

pub trait AsStringConverter {
  fn to_string(&self) -> Result<String>;
  fn from_str(s: &str) -> Result<Self>
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
  S: Fallible<Error = Error> + Writer + ?Sized,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver> {
    let value = field.to_string()?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(AsStringResolver { value, inner })
  }
}

impl<T, D> DeserializeWith<ArchivedString, T, D> for AsString
where
  T: AsStringConverter,
  D: Fallible<Error = Error> + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<T> {
    AsStringConverter::from_str(field.as_str())
  }
}

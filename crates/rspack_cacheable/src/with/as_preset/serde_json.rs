use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use serde_json::{Map, Value};

use super::AsPreset;
use crate::{Error, Result};

pub struct SerdeJsonResolver {
  inner: StringResolver,
  value: String,
}

// for Value
impl ArchiveWith<Value> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = SerdeJsonResolver;

  #[inline]
  fn resolve_with(_field: &Value, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let SerdeJsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<S> SerializeWith<Value, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer,
{
  #[inline]
  fn serialize_with(field: &Value, serializer: &mut S) -> Result<Self::Resolver> {
    let value = serde_json::to_string(field)
      .map_err(|_| Error::MessageError("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(SerdeJsonResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, Value, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Value> {
    serde_json::from_str(field)
      .map_err(|_| Error::MessageError("deserialize serde_json value failed"))
  }
}

// for Map<String, Value>
impl ArchiveWith<Map<String, Value>> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = SerdeJsonResolver;

  #[inline]
  fn resolve_with(
    _field: &Map<String, Value>,
    resolver: Self::Resolver,
    out: Place<Self::Archived>,
  ) {
    let SerdeJsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<S> SerializeWith<Map<String, Value>, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer,
{
  #[inline]
  fn serialize_with(field: &Map<String, Value>, serializer: &mut S) -> Result<Self::Resolver> {
    let value = serde_json::to_string(field)
      .map_err(|_| Error::MessageError("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(SerdeJsonResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, Map<String, Value>, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Map<String, Value>> {
    serde_json::from_str(field)
      .map_err(|_| Error::MessageError("deserialize serde_json value failed"))
  }
}

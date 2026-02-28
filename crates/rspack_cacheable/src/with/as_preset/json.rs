use json::JsonValue;
use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{Error, Result};

pub struct JsonResolver {
  inner: StringResolver,
  value: String,
}

impl ArchiveWith<JsonValue> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = JsonResolver;

  #[inline]
  fn resolve_with(_field: &JsonValue, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let JsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<S> SerializeWith<JsonValue, S> for AsPreset
where
  S: Fallible<Error = Error> + Writer,
{
  #[inline]
  fn serialize_with(field: &JsonValue, serializer: &mut S) -> Result<Self::Resolver> {
    let value = json::stringify(field.clone());
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(JsonResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, JsonValue, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<JsonValue> {
    json::parse(field).map_err(|_| Error::MessageError("deserialize json value failed"))
  }
}

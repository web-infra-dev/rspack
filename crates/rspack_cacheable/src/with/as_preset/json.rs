use json::JsonValue;
use rkyv::{
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use super::AsPreset;
use crate::{DeserializeError, SerializeError};

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
  S: Fallible<Error = SerializeError> + Writer,
{
  #[inline]
  fn serialize_with(
    field: &JsonValue,
    serializer: &mut S,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = json::stringify(field.clone());
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(JsonResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, JsonValue, D> for AsPreset
where
  D: Fallible<Error = DeserializeError>,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<JsonValue, DeserializeError> {
    json::parse(field).map_err(|_| DeserializeError::MessageError("deserialize json value failed"))
  }
}

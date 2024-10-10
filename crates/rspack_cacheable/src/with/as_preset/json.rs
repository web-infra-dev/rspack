use json::JsonValue;
use rkyv::{
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use super::AsPreset;
use crate::{DeserializeError, Deserializer, SerializeError, Serializer};

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

impl<'a> SerializeWith<JsonValue, Serializer<'a>> for AsPreset {
  #[inline]
  fn serialize_with(
    field: &JsonValue,
    serializer: &mut Serializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = json::stringify(field.clone());
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(JsonResolver { value, inner })
  }
}

impl DeserializeWith<ArchivedString, JsonValue, Deserializer> for AsPreset {
  #[inline]
  fn deserialize_with(
    field: &ArchivedString,
    _: &mut Deserializer,
  ) -> Result<JsonValue, DeserializeError> {
    json::parse(field)
      .map_err(|_| DeserializeError::DeserializeFailed("deserialize json value failed"))
  }
}

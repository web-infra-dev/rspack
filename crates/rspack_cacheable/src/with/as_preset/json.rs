use json::JsonValue;
use rkyv::{
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

pub struct JsonResolver {
  inner: StringResolver,
  value: String,
}

impl ArchiveWith<JsonValue> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = JsonResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &JsonValue,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let JsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, pos, inner, out);
  }
}

impl SerializeWith<JsonValue, CacheableSerializer> for AsPreset {
  #[inline]
  fn serialize_with(
    field: &JsonValue,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = json::stringify(field.clone());
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(JsonResolver { value, inner })
  }
}

impl DeserializeWith<ArchivedString, JsonValue, CacheableDeserializer> for AsPreset {
  #[inline]
  fn deserialize_with(
    field: &ArchivedString,
    _: &mut CacheableDeserializer,
  ) -> Result<JsonValue, DeserializeError> {
    json::parse(field)
      .map_err(|_| DeserializeError::DeserializeFailed("deserialize json value failed"))
  }
}

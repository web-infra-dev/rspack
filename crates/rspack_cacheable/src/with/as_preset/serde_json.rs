use rkyv::{
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use serde_json::Value;

use super::AsPreset;
use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

pub struct SerdeJsonResolver {
  inner: StringResolver,
  value: String,
}

impl ArchiveWith<Value> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = SerdeJsonResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &Value,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let SerdeJsonResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, pos, inner, out);
  }
}

impl SerializeWith<Value, CacheableSerializer> for AsPreset {
  #[inline]
  fn serialize_with(
    field: &Value,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = serde_json::to_string(field)
      .map_err(|_| SerializeError::SerializeFailed("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(SerdeJsonResolver { value, inner })
  }
}

impl DeserializeWith<ArchivedString, Value, CacheableDeserializer> for AsPreset {
  #[inline]
  fn deserialize_with(
    field: &ArchivedString,
    _: &mut CacheableDeserializer,
  ) -> Result<Value, DeserializeError> {
    serde_json::from_str(field)
      .map_err(|_| DeserializeError::DeserializeFailed("deserialize serde_json value failed"))
  }
}

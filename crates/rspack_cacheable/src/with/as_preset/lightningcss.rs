use lightningcss::targets::Browsers;
use rkyv::{
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{CacheableDeserializer, CacheableSerializer, DeserializeError, SerializeError};

pub struct BrowsersResolver {
  inner: StringResolver,
  value: String,
}

impl ArchiveWith<Browsers> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = BrowsersResolver;

  #[inline]
  unsafe fn resolve_with(
    _field: &Browsers,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    let BrowsersResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, pos, inner, out);
  }
}

impl SerializeWith<Browsers, CacheableSerializer> for AsPreset {
  #[inline]
  fn serialize_with(
    field: &Browsers,
    serializer: &mut CacheableSerializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = serde_json::to_string(field)
      .map_err(|_| SerializeError::SerializeFailed("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(BrowsersResolver { value, inner })
  }
}

impl DeserializeWith<ArchivedString, Browsers, CacheableDeserializer> for AsPreset {
  fn deserialize_with(
    field: &ArchivedString,
    _deserializer: &mut CacheableDeserializer,
  ) -> Result<Browsers, DeserializeError> {
    serde_json::from_str(field.as_str())
      .map_err(|_| DeserializeError::DeserializeFailed("deserialize serde_json value failed"))
  }
}

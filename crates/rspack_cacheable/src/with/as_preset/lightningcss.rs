use lightningcss::targets::Browsers;
use rkyv::{
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};

use super::AsPreset;
use crate::{DeserializeError, Deserializer, SerializeError, Serializer};

pub struct BrowsersResolver {
  inner: StringResolver,
  value: String,
}

impl ArchiveWith<Browsers> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = BrowsersResolver;

  #[inline]
  fn resolve_with(_field: &Browsers, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let BrowsersResolver { inner, value } = resolver;
    ArchivedString::resolve_from_str(&value, inner, out);
  }
}

impl<'a> SerializeWith<Browsers, Serializer<'a>> for AsPreset {
  #[inline]
  fn serialize_with(
    field: &Browsers,
    serializer: &mut Serializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = serde_json::to_string(field)
      .map_err(|_| SerializeError::SerializeFailed("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(BrowsersResolver { value, inner })
  }
}

impl DeserializeWith<ArchivedString, Browsers, Deserializer> for AsPreset {
  fn deserialize_with(
    field: &ArchivedString,
    _deserializer: &mut Deserializer,
  ) -> Result<Browsers, DeserializeError> {
    serde_json::from_str(field.as_str())
      .map_err(|_| DeserializeError::DeserializeFailed("deserialize serde_json value failed"))
  }
}

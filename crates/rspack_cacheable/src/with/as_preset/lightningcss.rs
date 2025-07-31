use lightningcss::targets::Browsers;
use rkyv::{
  Place,
  rancor::Fallible,
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use super::AsPreset;
use crate::{DeserializeError, SerializeError};

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

impl<S> SerializeWith<Browsers, S> for AsPreset
where
  S: Fallible<Error = SerializeError> + Writer,
{
  #[inline]
  fn serialize_with(
    field: &Browsers,
    serializer: &mut S,
  ) -> Result<Self::Resolver, SerializeError> {
    let value = serde_json::to_string(field)
      .map_err(|_| SerializeError::MessageError("serialize serde_json value failed"))?;
    let inner = ArchivedString::serialize_from_str(&value, serializer)?;
    Ok(BrowsersResolver { value, inner })
  }
}

impl<D> DeserializeWith<ArchivedString, Browsers, D> for AsPreset
where
  D: Fallible<Error = DeserializeError>,
{
  fn deserialize_with(
    field: &ArchivedString,
    _deserializer: &mut D,
  ) -> Result<Browsers, DeserializeError> {
    serde_json::from_str(field.as_str())
      .map_err(|_| DeserializeError::MessageError("deserialize serde_json value failed"))
  }
}

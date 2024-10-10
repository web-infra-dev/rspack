use rkyv::{
  rancor::{Fallible, Source},
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};
use smol_str::SmolStr;

use super::AsPreset;

impl ArchiveWith<SmolStr> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  fn resolve_with(field: &SmolStr, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedString::resolve_from_str(field.as_str(), resolver, out);
  }
}

impl<S> SerializeWith<SmolStr, S> for AsPreset
where
  S: ?Sized + Fallible + Writer,
  S::Error: Source,
{
  #[inline]
  fn serialize_with(field: &SmolStr, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, SmolStr, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<SmolStr, D::Error> {
    Ok(SmolStr::from(field.as_str()))
  }
}

use rkyv::{
  ser::{ScratchSpace, Serializer},
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};
use smol_str::SmolStr;

use super::AsPreset;

impl ArchiveWith<SmolStr> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  unsafe fn resolve_with(
    field: &SmolStr,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedString::resolve_from_str(field.as_str(), pos, resolver, out);
  }
}

impl<S> SerializeWith<SmolStr, S> for AsPreset
where
  S: ?Sized + Serializer + ScratchSpace,
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

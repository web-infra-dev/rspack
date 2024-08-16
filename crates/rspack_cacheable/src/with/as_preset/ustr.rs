use rkyv::{
  ser::{ScratchSpace, Serializer},
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};
use ustr::Ustr;

use super::AsPreset;

impl ArchiveWith<Ustr> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  unsafe fn resolve_with(
    field: &Ustr,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedString::resolve_from_str(field.as_str(), pos, resolver, out);
  }
}

impl<S> SerializeWith<Ustr, S> for AsPreset
where
  S: ?Sized + Serializer + ScratchSpace,
{
  #[inline]
  fn serialize_with(field: &Ustr, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, Ustr, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Ustr, D::Error> {
    Ok(Ustr::from(field.as_str()))
  }
}

use camino::Utf8PathBuf;
use rkyv::{
  ser::{ScratchSpace, Serializer},
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Fallible,
};

use super::AsPreset;

impl ArchiveWith<Utf8PathBuf> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  unsafe fn resolve_with(
    field: &Utf8PathBuf,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedString::resolve_from_str(field.as_str(), pos, resolver, out);
  }
}

impl<S> SerializeWith<Utf8PathBuf, S> for AsPreset
where
  S: ?Sized + Serializer + ScratchSpace,
{
  #[inline]
  fn serialize_with(field: &Utf8PathBuf, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, Utf8PathBuf, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Utf8PathBuf, D::Error> {
    Ok(Utf8PathBuf::from(field.as_str()))
  }
}

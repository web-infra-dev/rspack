use rkyv::{
  rancor::{Fallible, Source},
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Place,
};
use swc_core::ecma::atoms::Atom;

use super::AsPreset;

impl ArchiveWith<Atom> for AsPreset {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  #[inline]
  fn resolve_with(field: &Atom, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedString::resolve_from_str(field.as_str(), resolver, out);
  }
}

impl<S> SerializeWith<Atom, S> for AsPreset
where
  S: ?Sized + Fallible + Writer,
  S::Error: Source,
{
  #[inline]
  fn serialize_with(field: &Atom, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<D> DeserializeWith<ArchivedString, Atom, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<Atom, D::Error> {
    Ok(Atom::from(field.as_str()))
  }
}

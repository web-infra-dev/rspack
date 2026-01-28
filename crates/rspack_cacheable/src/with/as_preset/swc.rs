use rkyv::{
  Place,
  rancor::{Fallible, Source},
  ser::{Allocator, Writer},
  string::{ArchivedString, StringResolver},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use swc_core::{atoms::Wtf8Atom, ecma::atoms::Atom};

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

impl ArchiveWith<Wtf8Atom> for AsPreset {
  type Archived = ArchivedVec<u8>;
  type Resolver = VecResolver;

  #[inline]
  fn resolve_with(field: &Wtf8Atom, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedVec::resolve_from_len(field.as_bytes().len(), resolver, out);
  }
}

impl<S> SerializeWith<Wtf8Atom, S> for AsPreset
where
  S: Fallible + Allocator + Writer + ?Sized,
  S::Error: Source,
{
  #[inline]
  fn serialize_with(field: &Wtf8Atom, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedVec::serialize_from_slice(field.as_bytes(), serializer)
  }
}

impl<D> DeserializeWith<ArchivedVec<u8>, Wtf8Atom, D> for AsPreset
where
  D: ?Sized + Fallible,
{
  #[inline]
  fn deserialize_with(field: &ArchivedVec<u8>, _: &mut D) -> Result<Wtf8Atom, D::Error> {
    Ok(unsafe { Wtf8Atom::from_bytes_unchecked(field) })
  }
}

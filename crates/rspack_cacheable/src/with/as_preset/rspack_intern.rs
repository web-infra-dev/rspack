use rspack_intern::{Atom, AtomMap, AtomSet, IndexAtomMap, IndexAtomSet};

use super::AsPreset;
use crate::{
  Result,
  rkyv::{
    Place,
    rancor::{Fallible, Source},
    ser::Writer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
  },
  with::{AsMapConverter, AsVecConverter},
};

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

macro_rules! impl_atom_set_converter {
  ($type:ty) => {
    impl AsVecConverter for $type {
      type Item = Atom;

      fn len(&self) -> usize {
        std::ops::Deref::deref(self).len()
      }

      fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        std::ops::Deref::deref(self).iter()
      }

      fn from(data: impl Iterator<Item = Result<Self::Item>>) -> Result<Self> {
        Ok(data.collect::<Result<Vec<_>>>()?.into_iter().collect())
      }
    }
  };
}

impl_atom_set_converter!(AtomSet);
impl_atom_set_converter!(IndexAtomSet);

macro_rules! impl_atom_map_converter {
  ($type:ident) => {
    impl<V> AsMapConverter for $type<V> {
      type Key = Atom;
      type Value = V;

      fn len(&self) -> usize {
        std::ops::Deref::deref(self).len()
      }

      fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
        std::ops::Deref::deref(self).iter()
      }

      fn from(data: impl Iterator<Item = Result<(Self::Key, Self::Value)>>) -> Result<Self> {
        Ok(data.collect::<Result<Vec<_>>>()?.into_iter().collect())
      }
    }
  };
}

impl_atom_map_converter!(AtomMap);
impl_atom_map_converter!(IndexAtomMap);

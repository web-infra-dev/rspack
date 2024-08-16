use std::marker::PhantomData;

use rkyv::{
  ser::{ScratchSpace, Serializer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, Fallible, Serialize,
};

use crate::{with::AsCacheable, CacheableDeserializer, DeserializeError};

struct RefWrapper<'o, A, O>(&'o O, PhantomData<A>);

impl<A: ArchiveWith<O>, O> Archive for RefWrapper<'_, A, O> {
  type Archived = A::Archived;
  type Resolver = A::Resolver;

  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    A::resolve_with(self.0, pos, resolver, out)
  }
}

impl<A, O, S> Serialize<S> for RefWrapper<'_, A, O>
where
  A: ArchiveWith<O> + SerializeWith<O, S>,
  S: Fallible + Serializer + ?Sized,
{
  fn serialize(&self, s: &mut S) -> Result<Self::Resolver, S::Error> {
    A::serialize_with(self.0, s)
  }
}

pub struct AsVec<T = AsCacheable> {
  _inner: T,
}

#[allow(clippy::len_without_is_empty)]
pub trait AsVecConverter {
  type Item;
  fn len(&self) -> usize;
  fn iter(&self) -> impl ExactSizeIterator<Item = &Self::Item>;
  fn from(
    data: impl ExactSizeIterator<Item = Result<Self::Item, DeserializeError>>,
  ) -> Result<Self, DeserializeError>
  where
    Self: Sized;
}

impl<T, O, A> ArchiveWith<T> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  A: ArchiveWith<O>,
{
  type Archived = ArchivedVec<A::Archived>;
  type Resolver = VecResolver;

  unsafe fn resolve_with(
    field: &T,
    pos: usize,
    resolver: Self::Resolver,
    out: *mut Self::Archived,
  ) {
    ArchivedVec::resolve_from_len(field.len(), pos, resolver, out)
  }
}

impl<T, A, O, S> SerializeWith<T, S> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  S: Fallible + ScratchSpace + Serializer + ?Sized,
  A: ArchiveWith<O> + SerializeWith<O, S>,
{
  fn serialize_with(field: &T, s: &mut S) -> Result<Self::Resolver, S::Error> {
    let iter = field
      .iter()
      .map(|value| RefWrapper::<'_, A, O>(value, PhantomData));

    ArchivedVec::serialize_from_iter(iter, s)
  }
}

impl<T, A, O> DeserializeWith<ArchivedVec<A::Archived>, T, CacheableDeserializer> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  A: ArchiveWith<O> + DeserializeWith<A::Archived, O, CacheableDeserializer>,
{
  fn deserialize_with(
    field: &ArchivedVec<A::Archived>,
    d: &mut CacheableDeserializer,
  ) -> Result<T, DeserializeError> {
    T::from(field.iter().map(|item| A::deserialize_with(item, d)))
  }
}

// for Vec
impl<T> AsVecConverter for Vec<T> {
  type Item = T;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl ExactSizeIterator<Item = &Self::Item> {
    <[T]>::iter(self)
  }
  fn from(
    data: impl ExactSizeIterator<Item = Result<Self::Item, DeserializeError>>,
  ) -> Result<Self, DeserializeError> {
    data.collect::<Result<Vec<_>, DeserializeError>>()
  }
}

// for HashSet
impl<T, S> AsVecConverter for std::collections::HashSet<T, S>
where
  T: std::cmp::Eq + std::hash::Hash,
  S: core::hash::BuildHasher + Default,
{
  type Item = T;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl ExactSizeIterator<Item = &Self::Item> {
    self.iter()
  }
  fn from(
    data: impl ExactSizeIterator<Item = Result<Self::Item, DeserializeError>>,
  ) -> Result<Self, DeserializeError> {
    data.collect::<Result<std::collections::HashSet<T, S>, DeserializeError>>()
  }
}

// for indexmap::IndexSet
impl<T, S> AsVecConverter for indexmap::IndexSet<T, S>
where
  T: std::cmp::Eq + std::hash::Hash,
  S: core::hash::BuildHasher + Default,
{
  type Item = T;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl ExactSizeIterator<Item = &Self::Item> {
    self.iter()
  }
  fn from(
    data: impl ExactSizeIterator<Item = Result<Self::Item, DeserializeError>>,
  ) -> Result<Self, DeserializeError> {
    data.collect::<Result<indexmap::IndexSet<T, S>, DeserializeError>>()
  }
}

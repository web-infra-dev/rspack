use std::marker::PhantomData;

use rkyv::{
  Archive, Place, Serialize,
  rancor::Fallible,
  ser::{Allocator, Writer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result, with::AsCacheable};

struct RefWrapper<'o, A, O>(&'o O, PhantomData<A>);

impl<A: ArchiveWith<O>, O> Archive for RefWrapper<'_, A, O> {
  type Archived = A::Archived;
  type Resolver = A::Resolver;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    A::resolve_with(self.0, resolver, out)
  }
}

impl<A, O, S> Serialize<S> for RefWrapper<'_, A, O>
where
  A: ArchiveWith<O> + SerializeWith<O, S>,
  S: Fallible + ?Sized,
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
  fn iter(&self) -> impl Iterator<Item = &Self::Item>;
  fn from(data: impl Iterator<Item = Result<Self::Item>>) -> Result<Self>
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

  fn resolve_with(field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedVec::resolve_from_len(field.len(), resolver, out)
  }
}

impl<T, A, O, S> SerializeWith<T, S> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  S: Fallible + Allocator + Writer + ?Sized,
  A: ArchiveWith<O> + SerializeWith<O, S>,
{
  fn serialize_with(field: &T, s: &mut S) -> Result<Self::Resolver, S::Error> {
    let iter = field
      .iter()
      .map(|value| RefWrapper::<'_, A, O>(value, PhantomData))
      .collect::<Vec<_>>();

    ArchivedVec::serialize_from_slice(&iter, s)
  }
}

impl<T, A, O, D> DeserializeWith<ArchivedVec<A::Archived>, T, D> for AsVec<A>
where
  T: AsVecConverter<Item = O>,
  D: Fallible<Error = Error> + ?Sized,
  A: ArchiveWith<O> + DeserializeWith<A::Archived, O, D>,
{
  fn deserialize_with(field: &ArchivedVec<A::Archived>, d: &mut D) -> Result<T> {
    T::from(field.iter().map(|item| A::deserialize_with(item, d)))
  }
}

// for Vec
impl<T> AsVecConverter for Vec<T> {
  type Item = T;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl Iterator<Item = &Self::Item> {
    <[T]>::iter(self)
  }
  fn from(data: impl Iterator<Item = Result<Self::Item>>) -> Result<Self> {
    data.collect::<Result<Vec<_>>>()
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
  fn iter(&self) -> impl Iterator<Item = &Self::Item> {
    self.iter()
  }
  fn from(data: impl Iterator<Item = Result<Self::Item>>) -> Result<Self> {
    data.collect::<Result<std::collections::HashSet<T, S>>>()
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
  fn iter(&self) -> impl Iterator<Item = &Self::Item> {
    self.iter()
  }
  fn from(data: impl Iterator<Item = Result<Self::Item>>) -> Result<Self> {
    data.collect::<Result<indexmap::IndexSet<T, S>>>()
  }
}

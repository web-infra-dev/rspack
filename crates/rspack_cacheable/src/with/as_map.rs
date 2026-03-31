use std::marker::PhantomData;

use rkyv::{
  Archive, Place, Serialize,
  collections::util::Entry as RkyvEntry,
  rancor::Fallible,
  ser::{Allocator, Writer},
  vec::{ArchivedVec, VecResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result, with::AsCacheable};

pub struct AsMap<WK = AsCacheable, WV = AsCacheable> {
  _key: WK,
  _value: WV,
}

#[allow(clippy::len_without_is_empty)]
pub trait AsMapConverter {
  type Key;
  type Value;
  fn len(&self) -> usize;
  fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)>;
  fn from(data: impl Iterator<Item = Result<(Self::Key, Self::Value)>>) -> Result<Self>
  where
    Self: Sized;
}

pub struct Entry<K, V, WK, WV> {
  key: K,
  value: V,
  // with for key
  _key: PhantomData<WK>,
  // with for value
  _value: PhantomData<WV>,
}

impl<WK, WV, K, V> Archive for Entry<&'_ K, &'_ V, WK, WV>
where
  WK: ArchiveWith<K>,
  WV: ArchiveWith<V>,
{
  type Archived = RkyvEntry<WK::Archived, WV::Archived>;
  type Resolver = (WK::Resolver, WV::Resolver);

  #[inline]
  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let field_ptr = unsafe { &raw mut (*out.ptr()).key };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    WK::resolve_with(self.key, resolver.0, field_out);
    let field_ptr = unsafe { &raw mut (*out.ptr()).value };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    WV::resolve_with(self.value, resolver.1, field_out);
  }
}

impl<WK, WV, K, V, S> Serialize<S> for Entry<&'_ K, &'_ V, WK, WV>
where
  WK: SerializeWith<K, S>,
  WV: SerializeWith<V, S>,
  S: Fallible + ?Sized,
{
  #[inline]
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    Ok((
      WK::serialize_with(self.key, serializer)?,
      WV::serialize_with(self.value, serializer)?,
    ))
  }
}

impl<T, K, V, WK, WV> ArchiveWith<T> for AsMap<WK, WV>
where
  T: AsMapConverter<Key = K, Value = V>,
  WK: ArchiveWith<K>,
  WV: ArchiveWith<V>,
{
  type Archived = ArchivedVec<RkyvEntry<WK::Archived, WV::Archived>>;
  type Resolver = VecResolver;

  fn resolve_with(field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedVec::resolve_from_len(field.len(), resolver, out)
  }
}

impl<T, K, V, WK, WV, S> SerializeWith<T, S> for AsMap<WK, WV>
where
  T: AsMapConverter<Key = K, Value = V>,
  WK: ArchiveWith<K>,
  WV: ArchiveWith<V>,
  S: Fallible + Allocator + Writer + ?Sized,
  for<'a> Entry<&'a K, &'a V, WK, WV>: Serialize<S>,
{
  fn serialize_with(field: &T, s: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedVec::serialize_from_slice(
      &field
        .iter()
        .map(|(key, value)| Entry {
          key,
          value,
          _key: PhantomData::<WK>::default(),
          _value: PhantomData::<WV>::default(),
        })
        .collect::<Vec<_>>(),
      s,
    )
  }
}

impl<K, V, WK, WV, T, D> DeserializeWith<ArchivedVec<RkyvEntry<WK::Archived, WV::Archived>>, T, D>
  for AsMap<WK, WV>
where
  T: AsMapConverter<Key = K, Value = V>,
  K: std::hash::Hash + Eq,
  D: Fallible<Error = Error> + ?Sized,
  WK: ArchiveWith<K> + DeserializeWith<WK::Archived, K, D>,
  WV: ArchiveWith<V> + DeserializeWith<WV::Archived, V, D>,
{
  fn deserialize_with(
    field: &ArchivedVec<RkyvEntry<WK::Archived, WV::Archived>>,
    deserializer: &mut D,
  ) -> Result<T> {
    T::from(field.iter().map(|RkyvEntry { key, value }| {
      Ok((
        WK::deserialize_with(key, deserializer)?,
        WV::deserialize_with(value, deserializer)?,
      ))
    }))
  }
}

// for HashMap
impl<K, V, S> AsMapConverter for std::collections::HashMap<K, V, S>
where
  K: std::cmp::Eq + std::hash::Hash,
  S: core::hash::BuildHasher + Default,
{
  type Key = K;
  type Value = V;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
    self.iter()
  }
  fn from(data: impl Iterator<Item = Result<(Self::Key, Self::Value)>>) -> Result<Self> {
    data.collect::<Result<std::collections::HashMap<K, V, S>>>()
  }
}

// for hashlink::LinkedHashMap
impl<K, V, S> AsMapConverter for hashlink::LinkedHashMap<K, V, S>
where
  K: std::cmp::Eq + std::hash::Hash,
  S: core::hash::BuildHasher + Default,
{
  type Key = K;
  type Value = V;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
    self.iter()
  }
  fn from(data: impl Iterator<Item = Result<(Self::Key, Self::Value)>>) -> Result<Self> {
    data.collect::<Result<hashlink::LinkedHashMap<K, V, S>>>()
  }
}

// for indexmap::IndexMap
impl<K, V, S> AsMapConverter for indexmap::IndexMap<K, V, S>
where
  K: std::cmp::Eq + std::hash::Hash,
  S: core::hash::BuildHasher + Default,
{
  type Key = K;
  type Value = V;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
    self.iter()
  }
  fn from(data: impl Iterator<Item = Result<(Self::Key, Self::Value)>>) -> Result<Self> {
    data.collect::<Result<indexmap::IndexMap<K, V, S>>>()
  }
}

// for DashMap
impl<K, V, S> AsMapConverter for dashmap::DashMap<K, V, S>
where
  K: std::cmp::Eq + std::hash::Hash,
  S: core::hash::BuildHasher + Clone + Default,
{
  type Key = K;
  type Value = V;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
    dashmap::DashMap::iter(self).map(|item| {
      let (key, value) = item.pair();
      let key: *const Self::Key = key;
      let value: *const Self::Value = value;
      // SAFETY: The key value lifetime should be equal with self
      unsafe { (&*key, &*value) }
    })
  }
  fn from(data: impl Iterator<Item = Result<(Self::Key, Self::Value)>>) -> Result<Self> {
    data.collect::<Result<dashmap::DashMap<K, V, S>>>()
  }
}

use std::{
  hash::{Hash, Hasher},
  ops::RangeBounds,
};

use indexmap::{
  map::{Drain, IntoIter, Iter, IterMut, Keys, Values},
  IndexMap,
};

use crate::{
  ChunkLoading, DependencyId, EntryOptions, Filename, LibraryOptions, PublicPath, Reflectable,
  Reflector, Root,
};

#[derive(Debug, Default)]
pub struct Entries {
  #[cfg(feature = "napi")]
  reflector: Reflector,
  i: IndexMap<String, Root<EntryData>>,
}

impl Entries {
  pub fn new() -> Self {
    Self {
      #[cfg(feature = "napi")]
      reflector: Reflector::default(),
      i: IndexMap::new(),
    }
  }

  #[inline]
  pub fn insert(&mut self, key: String, value: Root<EntryData>) -> Option<Root<EntryData>> {
    self.i.insert(key, value)
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.i.len()
  }

  #[inline]
  pub fn contains_key(&self, key: &str) -> bool {
    self.i.contains_key(key)
  }

  #[inline]
  pub fn swap_remove(&mut self, key: &str) -> Option<Root<EntryData>> {
    self.i.swap_remove(key)
  }

  #[inline]
  pub fn keys(&self) -> Keys<'_, String, Root<EntryData>> {
    self.i.keys()
  }

  #[inline]
  pub fn values(&self) -> Values<'_, String, Root<EntryData>> {
    self.i.values()
  }

  #[inline]
  pub fn drain<R>(&mut self, range: R) -> Drain<'_, String, Root<EntryData>>
  where
    R: RangeBounds<usize>,
  {
    self.i.drain(range)
  }

  #[inline]
  pub fn get(&self, key: &str) -> Option<&Root<EntryData>> {
    self.i.get(key)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &str) -> Option<&mut Root<EntryData>> {
    self.i.get_mut(key)
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.i.is_empty()
  }

  #[inline]
  pub fn get_index_of(&self, key: &str) -> Option<usize> {
    self.i.get_index_of(key)
  }

  #[inline]
  pub fn iter(&self) -> Iter<String, Root<EntryData>> {
    self.i.iter()
  }
}

impl<'a> IntoIterator for &'a Entries {
  type Item = (&'a String, &'a Root<EntryData>);
  type IntoIter = Iter<'a, String, Root<EntryData>>;

  fn into_iter(self) -> Self::IntoIter {
    self.i.iter()
  }
}

impl<'a> IntoIterator for &'a mut Entries {
  type Item = (&'a String, &'a mut Root<EntryData>);
  type IntoIter = IterMut<'a, String, Root<EntryData>>;

  fn into_iter(self) -> Self::IntoIter {
    self.i.iter_mut()
  }
}

impl IntoIterator for Entries {
  type Item = (String, Root<EntryData>);
  type IntoIter = IntoIter<String, Root<EntryData>>;

  fn into_iter(self) -> Self::IntoIter {
    self.i.into_iter()
  }
}

#[cfg(feature = "napi")]
impl Reflectable for Entries {
  fn reflector(&self) -> &Reflector {
    &self.reflector
  }

  fn reflector_mut(&mut self) -> &mut Reflector {
    &mut self.reflector
  }
}

pub type EntryItem = Vec<String>;

#[derive(Debug, Clone, Default)]
pub struct EntryDescription {
  pub import: Option<EntryItem>,
  pub runtime: Option<String>,
  pub chunk_loading: Option<ChunkLoading>,
  pub async_chunks: Option<bool>,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub filename: Option<Filename>,
  pub depend_on: Option<Vec<String>>,
  pub library: Option<LibraryOptions>,
}

impl<V> From<V> for EntryDescription
where
  V: Into<String>,
{
  fn from(value: V) -> Self {
    Self {
      import: Some(vec![value.into()]),
      ..Default::default()
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct EntryData {
  #[cfg(feature = "napi")]
  reflector: Reflector,
  pub dependencies: Vec<DependencyId>,
  pub include_dependencies: Vec<DependencyId>,
  pub options: EntryOptions,
}

impl EntryData {
  pub fn new(
    dependencies: Vec<DependencyId>,
    include_dependencies: Vec<DependencyId>,
    options: EntryOptions,
  ) -> Self {
    Self {
      #[cfg(feature = "napi")]
      reflector: Reflector::default(),
      dependencies,
      include_dependencies,
      options,
    }
  }

  pub fn all_dependencies(&self) -> impl Iterator<Item = &DependencyId> {
    self
      .dependencies
      .iter()
      .chain(self.include_dependencies.iter())
  }
}

impl Hash for EntryData {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dependencies.hash(state);
    self.include_dependencies.hash(state);
    self.options.hash(state);
  }
}

impl PartialEq for EntryData {
  fn eq(&self, other: &Self) -> bool {
    self.dependencies == other.dependencies
      && self.include_dependencies == other.include_dependencies
      && self.options == other.options
  }
}

impl Eq for EntryData {}

#[cfg(feature = "napi")]
impl Reflectable for EntryData {
  fn reflector(&self) -> &Reflector {
    &self.reflector
  }

  fn reflector_mut(&mut self) -> &mut Reflector {
    &mut self.reflector
  }
}

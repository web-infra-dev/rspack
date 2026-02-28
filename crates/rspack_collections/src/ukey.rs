use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::{BuildHasherDefault, Hash},
};

use dashmap::{DashMap, DashSet};
use indexmap::{IndexMap, IndexSet};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! impl_item_ukey {
  ($type:ty) => {
    impl $crate::ItemUkey for $type {
      fn ukey(&self) -> $crate::Ukey {
        self.0
      }
    }
  };
}

pub type UkeyMap<K, V> = HashMap<K, V, BuildHasherDefault<UkeyHasher>>;
pub type UkeyIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<UkeyHasher>>;
pub type UkeyDashMap<K, V> = DashMap<K, V, BuildHasherDefault<UkeyHasher>>;

pub type UkeySet<K> = HashSet<K, BuildHasherDefault<UkeyHasher>>;
pub type UkeyIndexSet<K> = IndexSet<K, BuildHasherDefault<UkeyHasher>>;
pub type UkeyDashSet<K> = DashSet<K, BuildHasherDefault<UkeyHasher>>;

pub trait ItemUkey {
  fn ukey(&self) -> Ukey;
}

/// Ukey stands for Unique key
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct Ukey(u32);

impl Ukey {
  pub fn new(key: u32) -> Self {
    Self(key)
  }

  pub fn as_u32(&self) -> u32 {
    self.0
  }
}

impl From<u32> for Ukey {
  fn from(value: u32) -> Self {
    Self(value)
  }
}

impl From<Ukey> for u32 {
  fn from(value: Ukey) -> Self {
    value.0
  }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UkeyHasher(u32);

impl std::hash::Hasher for UkeyHasher {
  fn write(&mut self, _bytes: &[u8]) {
    unimplemented!("UkeyHasher should only used for UKey")
  }

  fn write_u32(&mut self, i: u32) {
    self.0 = i;
  }

  fn finish(&self) -> u64 {
    self.0 as u64
  }
}

pub trait DatabaseItem
where
  Self: Sized,
{
  type ItemUkey;
  fn ukey(&self) -> Self::ItemUkey;
}

pub struct Database<Item: DatabaseItem> {
  inner: HashMap<<Item as DatabaseItem>::ItemUkey, Item, BuildHasherDefault<UkeyHasher>>,
}

impl<Item: DatabaseItem> Debug for Database<Item> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Database").finish()
  }
}

impl<Item: DatabaseItem> Default for Database<Item> {
  fn default() -> Self {
    Self::new()
  }
}

impl<Item> Clone for Database<Item>
where
  Item: DatabaseItem + Clone,
  <Item as DatabaseItem>::ItemUkey: Clone,
{
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<Item: DatabaseItem> Database<Item> {
  pub fn new() -> Self {
    Self {
      inner: Default::default(),
    }
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}

impl<Item: DatabaseItem> Database<Item>
where
  <Item as DatabaseItem>::ItemUkey: Eq + Hash + Debug,
{
  pub fn contains(&self, id: &<Item as DatabaseItem>::ItemUkey) -> bool {
    self.inner.contains_key(id)
  }

  pub fn remove(&mut self, id: &<Item as DatabaseItem>::ItemUkey) -> Option<Item> {
    self.inner.remove(id)
  }

  pub fn entry(
    &mut self,
    id: <Item as DatabaseItem>::ItemUkey,
  ) -> std::collections::hash_map::Entry<'_, <Item as DatabaseItem>::ItemUkey, Item> {
    self.inner.entry(id)
  }

  pub fn get_mut(&mut self, id: &<Item as DatabaseItem>::ItemUkey) -> Option<&mut Item> {
    self.inner.get_mut(id)
  }

  pub fn get_many_mut<const N: usize>(
    &mut self,
    ids: [&<Item as DatabaseItem>::ItemUkey; N],
  ) -> [Option<&mut Item>; N] {
    self.inner.get_disjoint_mut(ids)
  }

  pub fn get(&self, id: &<Item as DatabaseItem>::ItemUkey) -> Option<&Item> {
    self.inner.get(id)
  }

  pub fn expect_get(&self, id: &<Item as DatabaseItem>::ItemUkey) -> &Item {
    self
      .inner
      .get(id)
      .unwrap_or_else(|| panic!("Item({id:?}) not found in Database"))
  }

  pub fn expect_get_mut(&mut self, id: &<Item as DatabaseItem>::ItemUkey) -> &mut Item {
    self
      .inner
      .get_mut(id)
      .unwrap_or_else(|| panic!("Item({id:?}) not found in Database"))
  }

  pub fn values(&self) -> impl Iterator<Item = &Item> {
    self.inner.values()
  }

  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Item> {
    self.inner.values_mut()
  }

  pub fn iter(&self) -> impl Iterator<Item = (&<Item as DatabaseItem>::ItemUkey, &Item)> {
    self.inner.iter()
  }

  pub fn iter_mut(
    &mut self,
  ) -> impl Iterator<Item = (&<Item as DatabaseItem>::ItemUkey, &mut Item)> {
    self.inner.iter_mut()
  }

  pub fn keys(&self) -> impl Iterator<Item = &<Item as DatabaseItem>::ItemUkey> {
    self.inner.keys()
  }

  pub fn into_items(self) -> impl Iterator<Item = Item> {
    self.inner.into_values()
  }
}

impl<Item: 'static + Sync + DatabaseItem> Database<Item>
where
  <Item as DatabaseItem>::ItemUkey: Eq + Hash + Debug + Sync,
{
  pub fn par_keys(&self) -> impl ParallelIterator<Item = &<Item as DatabaseItem>::ItemUkey> {
    self.keys().par_bridge()
  }

  pub fn par_values(&self) -> impl ParallelIterator<Item = &Item> {
    self.values().par_bridge()
  }
}

impl<Item: 'static + Send + DatabaseItem> Database<Item>
where
  <Item as DatabaseItem>::ItemUkey: Eq + Hash + Debug + Send,
{
  pub fn par_values_mut(&mut self) -> impl ParallelIterator<Item = &mut Item> {
    self.values_mut().par_bridge()
  }
}

impl<Item: DatabaseItem> Database<Item>
where
  <Item as DatabaseItem>::ItemUkey: Eq + Hash + Debug,
{
  pub fn add(&mut self, item: Item) -> &mut Item {
    debug_assert!(!self.inner.contains_key(&item.ukey()));
    let ukey = item.ukey();
    self.inner.entry(ukey).or_insert(item)
  }
}

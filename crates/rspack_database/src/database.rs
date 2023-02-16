use std::{any::Any, fmt::Debug};

use rustc_hash::FxHashMap;

use super::ukey::Ukey;
use crate::DatabaseItem;

#[derive(Clone)]
pub struct Database<Item> {
  inner: FxHashMap<Ukey<Item>, Item>,
}

impl<Item> Debug for Database<Item> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Database").finish()
  }
}

impl<Item: Any> Default for Database<Item> {
  fn default() -> Self {
    Self::new()
  }
}

impl<Item: Any> Database<Item> {
  pub fn new() -> Self {
    Self {
      inner: Default::default(),
    }
  }

  pub fn get(&self, id: &Ukey<Item>) -> Option<&Item> {
    self.inner.get(id)
  }

  pub fn get_mut(&mut self, id: &Ukey<Item>) -> Option<&mut Item> {
    self.inner.get_mut(id)
  }

  pub fn remove(&mut self, id: &Ukey<Item>) -> Option<Item> {
    self.inner.remove(id)
  }

  pub fn entry(&mut self, id: Ukey<Item>) -> std::collections::hash_map::Entry<Ukey<Item>, Item> {
    self.inner.entry(id)
  }

  pub fn expect_get(&self, id: &Ukey<Item>) -> &Item {
    self
      .inner
      .get(id)
      .unwrap_or_else(|| panic!("Not found {id:?}"))
  }
  pub fn expect_mut(&mut self, id: &Ukey<Item>) -> &mut Item {
    self
      .inner
      .get_mut(id)
      .unwrap_or_else(|| panic!("Not found {id:?}"))
  }

  pub fn values(&self) -> impl Iterator<Item = &Item> {
    self.inner.values()
  }

  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Item> {
    self.inner.values_mut()
  }

  pub fn iter(&self) -> impl Iterator<Item = (&Ukey<Item>, &Item)> {
    self.inner.iter()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Ukey<Item>, &mut Item)> {
    self.inner.iter_mut()
  }

  pub fn keys(&self) -> impl Iterator<Item = &Ukey<Item>> {
    self.inner.keys()
  }

  pub fn _todo_should_remove_this_method_inner_mut(&mut self) -> &mut FxHashMap<Ukey<Item>, Item> {
    &mut self.inner
  }

  pub fn into_items(self) -> impl Iterator<Item = Item> {
    self.inner.into_values()
  }
}

impl<Item: Default + DatabaseItem + 'static> Database<Item> {
  pub fn create_default_item(&mut self) -> &mut Item {
    let item = Item::default();
    let ukey = item.ukey();
    self.add(item);
    self.expect_mut(&ukey)
  }
}

impl<Item: DatabaseItem> Database<Item> {
  pub fn add(&mut self, item: Item) {
    debug_assert!(self.inner.get(&item.ukey()).is_none());
    let ukey = item.ukey();
    self.inner.insert(ukey, item);
  }
}

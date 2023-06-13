use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use rspack_identifier::{Identifier, IdentifierHasher};

use super::Storage;

#[derive(Debug)]
pub struct MemoryStorage<Item> {
  data: DashMap<Identifier, Item, BuildHasherDefault<IdentifierHasher>>,
}

impl<Item> MemoryStorage<Item> {
  pub fn new() -> Self {
    Self {
      data: DashMap::default(),
    }
  }
}

impl<Item> Storage<Item> for MemoryStorage<Item>
where
  Item: Clone + std::fmt::Debug + Send + Sync,
{
  fn get(&self, id: &Identifier) -> Option<Item> {
    self.data.get(id).map(|item| item.clone())
  }
  fn set(&self, id: Identifier, data: Item) {
    self.data.insert(id, data);
  }
  fn remove(&self, id: &Identifier) {
    self.data.remove(id);
  }
}

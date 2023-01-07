use dashmap::DashMap;

use super::Storage;
use crate::Identifier;

#[derive(Debug)]
pub struct MemoryStorage<Item> {
  data: DashMap<Identifier, Item>,
}

impl<Item> MemoryStorage<Item> {
  pub fn new() -> Self {
    Self {
      data: DashMap::new(),
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
}

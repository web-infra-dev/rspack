use dashmap::DashMap;

use super::Storage;

#[derive(Debug)]
pub struct MemoryStorage<Item> {
  data: DashMap<String, Item>,
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
  fn get(&self, id: &str) -> Option<Item> {
    self.data.get(id).map(|item| item.clone())
  }
  fn set(&self, id: String, data: Item) {
    self.data.insert(id, data);
  }
}

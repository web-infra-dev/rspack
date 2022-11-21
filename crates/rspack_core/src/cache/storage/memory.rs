use super::Storage;
use hashbrown::HashMap;

#[derive(Debug)]
pub struct MemoryStorage<Item> {
  data: HashMap<String, Item>,
}

impl<Item> MemoryStorage<Item> {
  pub fn new() -> Self {
    Self {
      data: HashMap::new(),
    }
  }
}

impl<Item> Storage<Item> for MemoryStorage<Item>
where
  Item: Clone + std::fmt::Debug,
{
  fn get(&self, id: &str) -> Option<Item> {
    self.data.get(id).cloned()
  }
  fn set(&mut self, id: String, data: Item) {
    self.data.insert(id, data);
  }
}

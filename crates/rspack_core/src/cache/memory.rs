use super::Cache;
use hashbrown::HashMap;

pub struct MemoryCache<Item> {
  data: HashMap<String, (Option<String>, Item)>,
}

impl<Item: Clone> Cache<Item> for MemoryCache<Item> {
  fn get(&self, id: String, etag: Option<String>) -> Option<Item> {
    self.data.get(&id).and_then(|item| {
      if item.0 == etag {
        Some(item.1.clone())
      } else {
        None
      }
    })
  }
  fn set(&mut self, id: String, etag: Option<String>, data: Item) {
    self.data.insert(id, (etag, data));
  }
}

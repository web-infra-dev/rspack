use std::fmt::Debug;

use rspack_collections::Identifier;

use crate::CacheOptions;

mod memory;
use memory::MemoryStorage;

pub trait Storage<Item>: Debug + Send + Sync {
  fn get(&self, id: &Identifier) -> Option<Item>;
  fn set(&self, id: Identifier, data: Item);
  fn remove(&self, id: &Identifier);
  fn begin_idle(&self);
  // fn end_idle(&self);
  // fn clear(&self);
}

pub fn new_storage<Item>(options: &CacheOptions) -> Option<Box<dyn Storage<Item>>>
where
  Item: Debug + Clone + Send + Sync + 'static,
{
  match options {
    CacheOptions::Disabled => None,
    CacheOptions::Memory { max_generations } => {
      Some(Box::new(MemoryStorage::new(max_generations.unwrap_or(1))))
    }
  }
}

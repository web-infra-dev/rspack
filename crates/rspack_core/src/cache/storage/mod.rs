use crate::CacheOptions;
use std::fmt::Debug;

mod memory;
use memory::MemoryStorage;

pub trait Storage<Item>: Debug + Send + Sync {
  fn get(&self, id: &str) -> Option<Item>;
  fn set(&self, id: String, data: Item);
  // fn begin_idle(&self);
  // fn end_idle(&self);
  // fn clear(&self);
}

pub fn new_storage<Item>(options: &CacheOptions) -> Option<Box<dyn Storage<Item>>>
where
  Item: Debug + Clone + Send + Sync + 'static,
{
  match options {
    CacheOptions::Disabled => None,
    _ => Some(Box::new(MemoryStorage::new())),
  }
}

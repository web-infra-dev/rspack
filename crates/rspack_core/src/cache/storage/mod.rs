use crate::CacheOptions;
use std::fmt::Debug;

mod memory;
mod none;
use memory::MemoryStorage;
use none::NoneStorage;

pub trait Storage<Item>: Debug {
  fn get(&self, id: &str) -> Option<Item>;
  fn set(&mut self, id: String, data: Item);
  // fn begin_idle(&self);
  // fn end_idle(&self);
  // fn clear(&self);
}

pub fn new_storage<Item>(options: &CacheOptions) -> Box<dyn Storage<Item>>
where
  Item: Debug + Clone + 'static,
{
  match options {
    CacheOptions::Disabled => Box::new(NoneStorage::new()),
    _ => Box::new(MemoryStorage::new()),
  }
}

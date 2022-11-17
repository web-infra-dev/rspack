mod memory;

pub use memory::MemoryCache;

pub trait Cache<Item> {
  fn get(&self, id: String, etag: Option<String>) -> Option<Item>;
  fn set(&mut self, id: String, etag: Option<String>, data: Item);
  // fn begin_idle(&self);
  // fn end_idle(&self);
  // fn clear(&self);
}

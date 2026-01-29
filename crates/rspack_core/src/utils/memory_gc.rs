use std::sync::atomic::{AtomicU32, Ordering};

use dashmap::DashMap;
use rspack_collections::{Identifier, IdentifierDashMap};

#[derive(Debug, Hash, PartialEq, Eq)]
struct CacheData<Item> {
  item: Item,
  generation: u32,
}

impl<Item> CacheData<Item> {
  fn new(item: Item, generation: u32) -> Self {
    Self { item, generation }
  }
}
/// memory storage with garbage collection based on generations
#[derive(Debug)]
pub struct MemoryGCStorage<Item> {
  generation: AtomicU32,
  max_generations: u32,
  data: IdentifierDashMap<CacheData<Item>>,
}

impl<Item> MemoryGCStorage<Item> {
  pub fn new(max_generations: u32) -> Self {
    Self {
      generation: AtomicU32::new(0),
      max_generations,
      data: DashMap::default(),
    }
  }
}

impl<Item> MemoryGCStorage<Item>
where
  Item: Clone + std::fmt::Debug + Send + Sync,
{
  pub(crate) fn get(&self, id: Identifier) -> Option<Item> {
    self.data.get_mut(&id).map(|mut item| {
      // Reset the generation to the current generation if the item is accessed
      item.generation = self.generation.load(Ordering::Relaxed);
      item.item.clone()
    })
  }
  pub(crate) fn set(&self, id: Identifier, data: Item) {
    self.data.insert(
      id,
      CacheData::new(data, self.generation.load(Ordering::Relaxed)),
    );
  }
  /// notify storage that the current generation is over and start a new one
  pub(crate) fn start_next_generation(&self) {
    let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
    self.data.retain(|_, cache_data| {
      // Remove the data if it is not accessed for `max_generations`.
      // With `max_generations` set to x, the cache was generated on generation y, will be removed on generation x + y + 1.
      //
      // For example:
      // Cache created on generation 0 will be removed on generation 2 with `max_generations` set to 1,
      // If it's not accessed on generation 1.
      cache_data.generation.saturating_add(self.max_generations) >= generation
    });
  }
}

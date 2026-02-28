use crate::cache::persistent::PersistentCacheOptions;

#[derive(Debug, Clone)]
pub enum CacheOptions {
  Disabled,
  Memory {
    /// The maximum number of generations to keep in memory.
    ///
    /// For example, if `max_generations` is set to 1,
    /// the cache will be removed if it's not accessed for 1 compilation generation.
    max_generations: u32,
  },
  Persistent(PersistentCacheOptions),
}

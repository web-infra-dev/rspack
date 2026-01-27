use crate::cache::persistent::PersistentCacheOptions;

#[derive(Debug, Clone)]
pub enum CacheOptions {
  Disabled,
  Memory,
  Persistent(PersistentCacheOptions),
}

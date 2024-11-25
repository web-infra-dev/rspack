use crate::cache::persistent::PersistentCacheOptions;

#[derive(Debug)]
pub enum CacheOptions {
  Disabled,
  Memory,
  Persistent(PersistentCacheOptions),
}

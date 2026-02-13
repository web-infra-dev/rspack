use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

/// Generate exist check hash from keys
pub fn exist_check_hash<K: AsRef<[u8]>>(keys: &[K]) -> u64 {
  let mut hasher = DefaultHasher::new();
  for key in keys {
    key.as_ref().hash(&mut hasher);
  }
  hasher.finish()
}

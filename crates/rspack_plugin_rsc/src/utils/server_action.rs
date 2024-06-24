use std::hash::{Hash, Hasher};

use rspack_hash::RspackHash;

pub fn generate_action_id(resource_path: &str, name: &str) -> u64 {
  let id = format!("{}:{}", resource_path, name);
  let mut s = RspackHash::new(&rspack_hash::HashFunction::Xxhash64);
  id.hash(&mut s);
  s.finish()
}

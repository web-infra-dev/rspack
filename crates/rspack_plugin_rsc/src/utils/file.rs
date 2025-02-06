use std::hash::Hash;

use rspack_hash::{HashDigest, HashFunction, RspackHash};

pub fn generate_asset_version(content_str: &str) -> String {
  let mut hasher = RspackHash::new(&HashFunction::Xxhash64);
  content_str.hash(&mut hasher);
  let hash_digest = hasher.digest(&HashDigest::Hex);
  let content_hash_str = hash_digest.rendered(8);
  content_hash_str.into()
}

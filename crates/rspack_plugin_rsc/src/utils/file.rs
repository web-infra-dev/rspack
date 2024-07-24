use std::hash::Hash;

use rspack_hash::{HashDigest, HashFunction, RspackHash};

use super::shared_data::ASSETS_HASH;

pub fn generate_asset_version(content_str: &str) -> String {
  let mut hasher = RspackHash::new(&HashFunction::Xxhash64);
  content_str.hash(&mut hasher);
  let hash_digest = hasher.digest(&HashDigest::Hex);
  let content_hash_str = hash_digest.rendered(8);
  content_hash_str.into()
}

// TODO: a better way to prevent write same content manifest?
pub async fn is_same_asset(filename: &str, content_str: &str) -> bool {
  let mut hasher = RspackHash::new(&HashFunction::Xxhash64);
  content_str.hash(&mut hasher);
  let hash_digest = hasher.digest(&HashDigest::Hex);
  let content_hash_str = hash_digest.rendered(8);
  let mut assets_hash = ASSETS_HASH.write().await;
  let prev_content_hash = assets_hash.get(filename);
  let is_same = match prev_content_hash {
    Some(prev_content_hash) => prev_content_hash.eq(content_hash_str),
    None => false,
  };
  assets_hash.insert(filename.into(), content_hash_str.into());
  is_same
}

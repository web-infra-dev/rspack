use std::hash::Hasher;

use rspack_hash::{HashDigest, HashFunction, RspackHash};

pub(crate) fn generate_debug_id(filename: &str, source: &[u8]) -> String {
  let mut hasher = RspackHash::new(&HashFunction::Xxhash64);
  hasher.write(source);
  let source_hash = hasher.digest(&HashDigest::Hex);

  let mut file_hasher = RspackHash::new(&HashFunction::Xxhash64);
  file_hasher.write(filename.as_bytes());
  file_hasher.write(source_hash.encoded().as_bytes());
  let file_hash = file_hasher.digest(&HashDigest::Hex);
  let hash128 = format!("{}{}", source_hash.encoded(), file_hash.encoded());

  let part3 = format!("4{}", &hash128[12..15]);
  let part4 = format!(
    "{:x}{}",
    u8::from_str_radix(&hash128[15..16], 16).unwrap_or(0) & 3 | 8,
    &hash128[17..20]
  );

  [
    &hash128[0..8],
    &hash128[8..12],
    &part3,
    &part4,
    &hash128[20..32],
  ]
  .join("-")
}

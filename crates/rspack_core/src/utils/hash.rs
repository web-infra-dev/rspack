use xxhash_rust::xxh3::xxh3_64;

pub fn get_xxh3_64_hash(text: &str) -> u64 {
  xxh3_64(text.as_bytes())
}

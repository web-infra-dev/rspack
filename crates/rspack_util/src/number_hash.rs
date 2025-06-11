/*
  MIT License http://www.opensource.org/licenses/mit-license.php
  Author Tobias Koppers @sokra
*/

// Port from https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util/numberHash.js

const FNV_64_THRESHOLD: usize = 1 << 24;

const FNV_OFFSET_32: u32 = 2166136261;

const FNV_PRIME_32: u32 = 16777619;

const MASK_31: u32 = 0x7fffffff;

const FNV_OFFSET_64: u64 = 0xCBF29CE484222325;

const FNV_PRIME_64: u64 = 0x100000001B3;

fn fnv1a32(s: &str) -> u32 {
  let mut hash = FNV_OFFSET_32;

  for code_unit in s.encode_utf16() {
    hash ^= code_unit as u32;
    hash = hash.wrapping_mul(FNV_PRIME_32);
  }

  hash & MASK_31
}

fn fnv1a64(s: &str) -> u64 {
  let mut hash = FNV_OFFSET_64;

  for code_unit in s.encode_utf16() {
    hash ^= code_unit as u64;
    hash = hash.wrapping_mul(FNV_PRIME_64);
  }

  hash
}

pub fn get_number_hash(s: &str, range: usize) -> usize {
  if range < FNV_64_THRESHOLD {
    (fnv1a32(s) as usize) % range
  } else {
    (fnv1a64(s) % (range as u64)) as usize
  }
}

#[test]
fn test_number_hash() {
  for n in [10, 100, 1000, 10000].iter() {
    let mut set = std::collections::HashSet::new();
    for i in 0..(*n * 200) {
      set.insert(get_number_hash(&format!("{i}"), *n));
      if set.len() >= (*n - 1) {
        break;
      }
    }
    assert_eq!(set.len(), (*n - 1));
  }

  let range_100_cases = vec![
    ("webpack", 88usize),
    ("rspack", 15usize),
    ("Hello, world!", 16usize),
    ("我能吞下玻璃而不伤身体", 97usize),
    ("君が笑ってると、僕もうれしくなるんだ。", 1usize),
    ("🤣👉🤡", 53usize),
  ];
  for (s, hash) in range_100_cases.iter() {
    assert_eq!(get_number_hash(s, 100), *hash);
  }

  let range_167772160_cases = vec![
    ("webpack", 125313516usize),
    ("rspack", 151778395usize),
    ("Hello, world!", 77102068usize),
    ("我能吞下玻璃而不伤身体", 63515641usize),
    ("君が笑ってると、僕もうれしくなるんだ。", 111837237usize),
    ("🤣👉🤡", 93616649usize),
  ];
  for (s, hash) in range_167772160_cases.iter() {
    assert_eq!(get_number_hash(s, 167772160), *hash);
  }
}

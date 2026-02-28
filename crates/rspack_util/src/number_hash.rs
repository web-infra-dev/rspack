/*
  MIT License http://www.opensource.org/licenses/mit-license.php
  Author Tobias Koppers @sokra
*/

// Port from https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util/numberHash.js

const FNV_64_THRESHOLD: usize = 1 << 24;

const FNV_OFFSET_32: u32 = 2_166_136_261;

const FNV_PRIME_32: u32 = 16_777_619;

const MASK_31: u32 = 0x7fff_ffff;

const FNV_OFFSET_64: u64 = 0xCBF2_9CE4_8422_2325;

const FNV_PRIME_64: u64 = 0x0100_0000_01B3;

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

  let test_cases = [
    // range = 100
    ("Hello, world!", 100usize, 16usize),
    (
      "You are right, but Rspack is a next-generation super fast bundler developed by the WebInfra team. Written in Rust, it make your build fly like rocket. With perfect support for Webpack ecosystem, it not only fast, but also friendly. Using Rspack, you can build faster and go home earlier.",
      100usize,
      73usize,
    ),
    ("我能吞下玻璃而不伤身体", 100usize, 97usize),
    ("君が笑ってると、僕もうれしくなるんだ。", 100usize, 1usize),
    ("🤣👉🤡", 100usize, 53usize),
    // range = 10 * (1 << 24)
    ("Hello, world!", 167_772_160_usize, 77_102_068_usize),
    (
      "You are right, but Rspack is a next-generation super fast bundler developed by the WebInfra team. Written in Rust, it make your build fly like rocket. With perfect support for Webpack ecosystem, it not only fast, but also friendly. Using Rspack, you can build faster and go home earlier.",
      167_772_160_usize,
      42_705_789_usize,
    ),
    (
      "我能吞下玻璃而不伤身体",
      167_772_160_usize,
      63_515_641_usize,
    ),
    (
      "君が笑ってると、僕もうれしくなるんだ。",
      167_772_160_usize,
      111_837_237_usize,
    ),
    ("🤣👉🤡", 167_772_160_usize, 93_616_649_usize),
  ];

  for (s, range, hash) in test_cases.iter() {
    assert_eq!(get_number_hash(s, *range), *hash);
  }
}

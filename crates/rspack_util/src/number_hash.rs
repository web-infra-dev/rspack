/*
  MIT License http://www.opensource.org/licenses/mit-license.php
  Author Tobias Koppers @sokra
*/

// From https://github.com/webpack/webpack/blob/v5.99.9/lib/util/numberHash.js

pub fn get_number_hash(str: &str, range: usize) -> usize {
  // We follow webpack using fnv1a, but it doesn't align with webpack's output
  // because webpack's code does not hash by byte as fnv algorithm.

  use std::hash::Hasher;

  let range = range as u64;
  let mut hasher = fnv::FnvHasher::default();
  hasher.write(str.as_bytes());
  (hasher.finish() % range) as usize
}

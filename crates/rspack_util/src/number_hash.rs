/*
  MIT License http://www.opensource.org/licenses/mit-license.php
  Author Tobias Koppers @sokra
*/

// Port from https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util/numberHash.js

const SAFE_LIMIT: usize = 2147483648usize;
const SAFE_PART: usize = SAFE_LIMIT - 1usize;
const COUNT: usize = 4usize;
// const arr: [usize; 5] = [0usize; 5];
const PRIMES: [usize; 4] = [3usize, 7usize, 17usize, 19usize];

#[allow(clippy::assign_op_pattern)]
pub fn get_number_hash(s: &str, range: usize) -> usize {
  let str = s.chars().collect::<Vec<_>>();
  let mut arr: [usize; 5] = [0; 5];
  let mut i = 0;
  while i < str.len() {
    let c = str[i];
    let mut j = 0;
    while j < COUNT {
      let p = (j + COUNT - 1) % COUNT;
      arr[j] = (arr[j] + (c as i32 as usize) * PRIMES[j] + arr[p]) % SAFE_PART;
      j += 1;
    }
    i += 1;

    let mut j = 0;
    while j < COUNT {
      let q = arr[j] % COUNT;
      arr[j] = arr[j] ^ (arr[q] >> 1);
      j += 1;
    }
  }

  if range <= SAFE_PART {
    let mut sum = 0;
    let mut j = 0;
    while j < COUNT {
      sum = (sum + arr[j]) % range;
      j += 1;
    }
    sum
  } else {
    let mut sum1 = 0;
    let mut sum2 = 0;
    let range_ext = usize::div_floor(range, SAFE_LIMIT);
    let mut j = 0;
    while j < COUNT {
      sum1 = (sum1 + arr[j]) % SAFE_PART;
      j += 1;
    }
    let mut j = 0;
    while j < COUNT {
      sum2 = (sum2 + arr[j]) % range_ext;
      j += 1;
    }

    (sum2 * SAFE_LIMIT + sum1) % range
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
}

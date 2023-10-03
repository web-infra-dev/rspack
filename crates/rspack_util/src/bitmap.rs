// If you have any idea on improving performance, please make a PR

use std::{borrow::Cow, vec};

// The reason why not using bitmaps crate is that it now supports max to 1024 bit size.
#[derive(Debug, Clone)]
pub enum BitMap {
  Small(u128),

  // when [000_000_11] grows, it becomes [000_000_11, 000_000_00], but
  // it actual means 000_000_00_000_000_11, store this way we can use
  // push to grow, instead of inserting elements in vector
  Large(Vec<u128>),
}

impl Default for BitMap {
  fn default() -> Self {
    Self::Small(0)
  }
}

impl From<usize> for BitMap {
  fn from(value: usize) -> Self {
    Self::small(value as u128)
  }
}

impl From<i32> for BitMap {
  fn from(value: i32) -> Self {
    debug_assert!(value >= 0);
    Self::small(value as u128)
  }
}

impl From<u128> for BitMap {
  fn from(value: u128) -> Self {
    Self::small(value)
  }
}

impl std::ops::BitOr for BitMap {
  type Output = BitMap;
  fn bitor(self, rhs: Self) -> BitMap {
    match (&self, &rhs) {
      (BitMap::Small(v), BitMap::Small(rhs)) => BitMap::Small(v | rhs),
      _ => {
        let orig = match &self {
          BitMap::Small(v) => Cow::Owned(vec![*v]),
          BitMap::Large(large) => Cow::Borrowed(large),
        };

        let rhs = match &rhs {
          BitMap::Small(v) => Cow::Owned(vec![*v]),
          BitMap::Large(large) => Cow::Borrowed(large),
        };

        let mut new_bitmap = vec![];

        let mut orig_idx = 0;
        let mut rhs_idx = 0;

        while orig_idx < orig.len() && rhs_idx < rhs.len() {
          let v = orig[orig_idx];
          let rhs = rhs[rhs_idx];

          new_bitmap.push(v | rhs);
          orig_idx += 1;
          rhs_idx += 1;
        }

        if orig_idx < orig.len() {
          new_bitmap.extend(orig[orig_idx..].iter());
        } else {
          new_bitmap.extend(rhs[rhs_idx..].iter());
        }

        BitMap::Large(new_bitmap)
      }
    }
  }
}

impl std::ops::BitOrAssign for BitMap {
  fn bitor_assign(&mut self, rhs: Self) {
    let orig = std::mem::take(self);
    let new_bitmap = orig | rhs;
    *self = new_bitmap;
  }
}

impl BitMap {
  pub fn small(v: u128) -> Self {
    Self::Small(v)
  }

  pub fn large(v: Vec<u128>) -> Self {
    Self::Large(v)
  }

  pub fn is_small(&self) -> bool {
    matches!(self, BitMap::Small(_))
  }

  pub fn expect_small(&self) -> u128 {
    match self {
      BitMap::Small(small) => *small,
      BitMap::Large(_) => unreachable!(),
    }
  }

  pub fn is_large(&self) -> bool {
    matches!(self, BitMap::Large(_))
  }

  pub fn expect_large(&self) -> &[u128] {
    match self {
      BitMap::Small(_) => unreachable!(),
      BitMap::Large(large) => large,
    }
  }

  pub fn contains(&self, v: &BitMap) -> bool {
    if self.is_small() && v.is_small() {
      // fast path
      let self_val = self.expect_small();
      let val = v.expect_small();

      (self_val | val) == self_val
    } else {
      // convert both to vector
      let self_vec = if self.is_small() {
        Cow::Owned(vec![self.expect_small()])
      } else {
        Cow::Borrowed(self.expect_large())
      };

      let other_vec = if v.is_small() {
        Cow::Owned(vec![v.expect_small()])
      } else {
        Cow::Borrowed(v.expect_large())
      };

      let mut self_idx = 0;
      let mut other_idx = 0;

      while self_idx < self_vec.len() && other_idx < other_vec.len() {
        let self_val = self_vec[self_idx];
        let other_val = other_vec[other_idx];

        if (self_val | other_val) != self_val {
          return false;
        }

        self_idx += 1;
        other_idx += 1;
      }

      if other_idx == other_vec.len() {
        // all values are checked
        true
      } else {
        // self_vec:  ________ 00011
        // other_vec: 00000000 00001
        // we checked all the self_vec, but not other_vec

        while other_idx < other_vec.len() {
          if other_vec[other_idx] != 0 {
            return false;
          }
          other_idx += 1;
        }

        true
      }
    }
  }

  // left shift one bit
  pub fn shift_left(&self) -> BitMap {
    match self {
      BitMap::Small(v) => {
        if v & (1u128 << 127) != 0 {
          BitMap::large(vec![*v << 1, 1u128])
        } else {
          BitMap::small(v << 1)
        }
      }
      BitMap::Large(large) => {
        let mut offset = 0;
        let mut new: Vec<_> = large
          .iter()
          .map(|v| {
            let mut res: u128 = v << 1;

            if offset > 0 {
              res |= 1;
              offset -= 1;
            }

            if v & (1u128 << 127) != 0 {
              offset += 1;
            }

            res
          })
          .collect();

        if offset > 0 {
          new.push(1u128);
        }

        BitMap::Large(new)
      }
    }
  }
}

#[test]
fn test_shift() {
  let v: BitMap = 1u128.into();
  assert_eq!(v.shift_left().expect_small(), 0b10);

  let v: BitMap = (1u128 << 127u128).into();
  assert!(v.is_small());
  let v = v.shift_left(); // << 1
  assert!(v.is_large()); // should grow
  assert_eq!(v.expect_large()[0], 0);
  assert_eq!(v.expect_large()[1], 1);

  let v: BitMap = BitMap::Large(vec![0, 0, 1 << 127]); // should grow correctly
  assert_eq!(v.shift_left().expect_large(), [0, 0, 0, 1]);

  let v: BitMap = BitMap::Large(vec![1 << 127, 1 << 127]); // should grow correctly
  assert_eq!(v.shift_left().expect_large(), [0, 1, 1]); // 100_100 -> 1_001_000 -> [000, 001, 1]

  let v: BitMap = BitMap::Large(vec![u128::MAX, u128::MAX]);
  // 111_111 -> 1_111_110 -> [110, 111, 1]
  assert_eq!(v.shift_left().expect_large(), [u128::MAX - 1, u128::MAX, 1]);
}

#[test]
fn test_contains() {
  // basic
  let v: BitMap = u128::MAX.into();
  assert!(v.contains(&1.into()));

  // large
  let v: BitMap = u128::MAX.into();
  let v = v.shift_left(); // 1_1110: [1110, 1]
  assert!(!v.contains(&1.into()));
  assert!(v.contains(&BitMap::Large(vec![0, 1])));
}

#[test]
fn test_bitor() {
  // small with small
  let v1: BitMap = 0b001.into();
  let v2: BitMap = 0b100.into();
  assert_eq!((v1 | v2).expect_small(), 0b101);

  // large with large
  let v1: BitMap = BitMap::large(vec![0b001]);
  let v2: BitMap = BitMap::large(vec![0b100]);
  assert_eq!((v1 | v2).expect_large(), vec![0b101]);

  // large with small
  let v1: BitMap = 0b001.into();
  let v2: BitMap = BitMap::large(vec![0, 1]);
  assert_eq!((v1 | v2).expect_large(), vec![1, 1]);

  // bitor assign
  let mut v1: BitMap = 0b001.into();
  let v2: BitMap = BitMap::large(vec![0, 1]);
  v1 |= v2;
  assert_eq!((v1).expect_large(), vec![1, 1]);
}

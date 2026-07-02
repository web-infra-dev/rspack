use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use smol_str::SmolStr;

use crate::{RspackHash, RspackHasher};

impl RspackHash for str {
  fn hash(&self, state: &mut RspackHasher) {
    state.write(self.as_bytes());
  }
}

impl RspackHash for String {
  fn hash(&self, state: &mut RspackHasher) {
    self.as_str().hash(state);
  }
}

impl RspackHash for SmolStr {
  fn hash(&self, state: &mut RspackHasher) {
    self.as_str().hash(state);
  }
}

impl RspackHash for Cow<'_, str> {
  fn hash(&self, state: &mut RspackHasher) {
    self.as_ref().hash(state);
  }
}

impl RspackHash for bool {
  fn hash(&self, state: &mut RspackHasher) {
    state.write(if *self { b"true" } else { b"false" });
  }
}

macro_rules! impl_content_hash_for_integer {
  ($($ty:ty),+ $(,)?) => {
    $(
      impl RspackHash for $ty {
        fn hash(&self, state: &mut RspackHasher) {
          let mut buffer = itoa::Buffer::new();
          state.write(buffer.format(*self).as_bytes());
        }
      }
    )+
  };
}

impl_content_hash_for_integer!(
  u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl RspackHash for Path {
  fn hash(&self, state: &mut RspackHasher) {
    self.to_string_lossy().hash(state);
  }
}

impl RspackHash for PathBuf {
  fn hash(&self, state: &mut RspackHasher) {
    self.as_path().hash(state);
  }
}

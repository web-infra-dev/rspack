use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use smol_str::SmolStr;

use crate::{RspackHash, RspackHashable};

impl RspackHashable for str {
  fn hash(&self, state: &mut RspackHash) {
    state.write(self.as_bytes());
  }
}

impl RspackHashable for String {
  fn hash(&self, state: &mut RspackHash) {
    self.as_str().hash(state);
  }
}

impl RspackHashable for SmolStr {
  fn hash(&self, state: &mut RspackHash) {
    self.as_str().hash(state);
  }
}

impl RspackHashable for Cow<'_, str> {
  fn hash(&self, state: &mut RspackHash) {
    self.as_ref().hash(state);
  }
}

impl RspackHashable for bool {
  fn hash(&self, state: &mut RspackHash) {
    state.write(if *self { b"true" } else { b"false" });
  }
}

macro_rules! impl_content_hash_for_integer {
  ($($ty:ty),+ $(,)?) => {
    $(
      impl RspackHashable for $ty {
        fn hash(&self, state: &mut RspackHash) {
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

impl RspackHashable for Path {
  fn hash(&self, state: &mut RspackHash) {
    self.to_string_lossy().hash(state);
  }
}

impl RspackHashable for PathBuf {
  fn hash(&self, state: &mut RspackHash) {
    self.as_path().hash(state);
  }
}

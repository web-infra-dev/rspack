use std::{
  hash::Hash,
  sync::atomic::{AtomicUsize, Ordering},
};

pub type ChunkUkey = Ukey;
pub type ChunkGroupUkey = Ukey;

static NEXT_UNIQUE_KEY: AtomicUsize = AtomicUsize::new(0);

/// Ukey stands for Unique key
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct Ukey(usize, Option<&'static str>);

impl Default for Ukey {
  fn default() -> Self {
    Self::new()
  }
}

impl Ukey {
  fn gen_ukey() -> usize {
    NEXT_UNIQUE_KEY.fetch_add(1, Ordering::Relaxed)
  }

  pub fn new() -> Self {
    Self(Self::gen_ukey(), None)
  }

  pub fn with_debug_info(info: &'static str) -> Self {
    Self(Self::gen_ukey(), Some(info))
  }

  pub fn ukey(&self) -> usize {
    self.0
  }
}

impl Hash for Ukey {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl PartialEq for Ukey {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

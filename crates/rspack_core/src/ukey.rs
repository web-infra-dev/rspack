use std::{
  hash::Hash,
  sync::atomic::{AtomicUsize, Ordering},
};

pub type ChunkUkey = Ukey;

static NEXT_UNIQUE_KEY: AtomicUsize = AtomicUsize::new(0);

/// Ukey stands for Unique key
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ukey(usize, Option<&'static str>);

impl Default for Ukey {
  fn default() -> Self {
    Self::new()
  }
}

impl Ukey {
  fn gen_ukey() -> usize {
    NEXT_UNIQUE_KEY.fetch_add(1, Ordering::SeqCst)
  }

  pub fn new() -> Self {
    Self(Self::gen_ukey(), None)
  }

  pub fn with_debug_info(info: &'static str) -> Self {
    Self(Self::gen_ukey(), Some(info))
  }
}

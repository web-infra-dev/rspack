use std::sync::atomic::{AtomicUsize, Ordering};

pub type ChunkUkey = Ukey;

static NEXT_RES_ID: AtomicUsize = AtomicUsize::new(0);

/// Ukey stands for Unique key
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ukey(usize, Option<&'static str>);

impl Default for Ukey {
  fn default() -> Self {
    Self::new()
  }
}

impl Ukey {
  pub fn new() -> Self {
    Self(NEXT_RES_ID.fetch_add(1, Ordering::SeqCst), None)
  }

  pub fn with_debug_info(info: &'static str) -> Self {
    Self(NEXT_RES_ID.fetch_add(1, Ordering::SeqCst), Some(info))
  }
}

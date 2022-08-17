use std::sync::atomic::{AtomicUsize, Ordering};

pub type ChunkRid = Rid;

static NEXT_RES_ID: AtomicUsize = AtomicUsize::new(0);

/// Rid stands for Resource Id
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rid(usize, Option<&'static str>);

impl Rid {
  pub fn new() -> Self {
    Self(NEXT_RES_ID.fetch_add(1, Ordering::SeqCst), None)
  }

  pub fn with_debug_info(info: &'static str) -> Self {
    Self(NEXT_RES_ID.fetch_add(1, Ordering::SeqCst), Some(info))
  }
}

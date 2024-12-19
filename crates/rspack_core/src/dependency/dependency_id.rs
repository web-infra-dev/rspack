use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

use rspack_cacheable::cacheable;
use serde::Serialize;

#[cacheable(hashable)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct DependencyId(u32);

pub static DEPENDENCY_ID: AtomicU32 = AtomicU32::new(0);

impl DependencyId {
  pub fn new() -> Self {
    Self(DEPENDENCY_ID.fetch_add(1, Relaxed))
  }
}

impl Default for DependencyId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::ops::Deref for DependencyId {
  type Target = u32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<u32> for DependencyId {
  fn from(id: u32) -> Self {
    Self(id)
  }
}

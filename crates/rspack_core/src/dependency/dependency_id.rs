use rspack_cacheable::cacheable;
use rspack_tasks::fetch_new_dependency_id;
use serde::Serialize;

#[cacheable(hashable)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct DependencyId(u32);

impl DependencyId {
  pub fn new() -> Self {
    let id = fetch_new_dependency_id();
    Self(id)
  }

  pub fn as_u32(&self) -> u32 {
    self.0
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

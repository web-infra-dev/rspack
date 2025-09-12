use std::{
  collections::{HashMap, HashSet},
  hash::{BuildHasherDefault, Hash},
};

use rspack_cacheable::cacheable;
use rspack_tasks::fetch_new_dependency_id;
use serde::Serialize;
use ustr::IdentityHasher;

const K: u32 = 0x93d765dd;
const ROTATE: u32 = 15;

#[cacheable(hashable)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct DependencyId(u32);

impl DependencyId {
  pub fn new() -> Self {
    let id = fetch_new_dependency_id();
    Self(id.wrapping_mul(K).rotate_left(ROTATE))
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

pub type DependencyIdMap<V> = HashMap<DependencyId, V, BuildHasherDefault<IdentityHasher>>;

pub type DependencyIdSet = HashSet<DependencyId, BuildHasherDefault<IdentityHasher>>;

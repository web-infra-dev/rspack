use std::{
  collections::{HashMap, HashSet},
  hash::{BuildHasherDefault, Hash, Hasher},
};

use rspack_cacheable::cacheable;
use rspack_tasks::fetch_new_dependency_id;
use rustc_hash::FxHasher;
use serde::Serialize;
use ustr::IdentityHasher;

#[cacheable(hashable)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct DependencyId(u64);

impl DependencyId {
  pub fn new() -> Self {
    let id = fetch_new_dependency_id();
    let mut hasher = FxHasher::default();
    id.hash(&mut hasher);
    Self(hasher.finish())
  }
}

impl Default for DependencyId {
  fn default() -> Self {
    Self::new()
  }
}

impl std::ops::Deref for DependencyId {
  type Target = u64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub type DependencyIdMap<V> = HashMap<DependencyId, V, BuildHasherDefault<IdentityHasher>>;

pub type DependencyIdSet = HashSet<DependencyId, BuildHasherDefault<IdentityHasher>>;

use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

use serde::Serialize;
use swc_core::ecma::atoms::Atom;

use crate::ModuleGraph;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct DependencyId(u32);

pub static DEPENDENCY_ID: AtomicU32 = AtomicU32::new(0);

impl DependencyId {
  pub fn new() -> Self {
    Self(DEPENDENCY_ID.fetch_add(1, Relaxed))
  }

  pub fn set_ids(&self, ids: Vec<Atom>, mg: &mut ModuleGraph) {
    mg.set_dep_meta(*self, ids);
  }

  /// # Panic
  /// This method will panic if one of following condition is true:
  /// * current dependency id is not belongs to `HarmonyImportSpecifierDependency` or  `HarmonyExportImportedSpecifierDependency`
  /// * current id is not in `ModuleGraph`
  pub fn get_ids(&self, mg: &ModuleGraph) -> Vec<Atom> {
    let dep = mg.dependency_by_id(self).expect("should have dep");
    dep.get_ids(mg)
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

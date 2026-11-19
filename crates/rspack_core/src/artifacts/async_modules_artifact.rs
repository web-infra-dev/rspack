use std::mem;

use rspack_collections::IdentifierSet;

use crate::{
  ArtifactExt, ModuleIdentifier,
  incremental::{Incremental, IncrementalPasses},
};

#[derive(Debug, Default)]
pub struct AsyncModulesArtifact {
  inferred: IdentifierSet,
  pub(crate) startup: IdentifierSet,
}

impl ArtifactExt for AsyncModulesArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::FINISH_MODULES;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(&mut new.inferred, &mut old.inferred);
    }
  }
}

impl AsyncModulesArtifact {
  pub(crate) fn contains(&self, module: &ModuleIdentifier) -> bool {
    self.inferred.contains(module) || self.startup.contains(module)
  }

  pub(crate) fn set_inferred(&mut self, module: ModuleIdentifier, is_async: bool) -> bool {
    if is_async {
      self.inferred.insert(module)
    } else {
      self.inferred.remove(&module)
    }
  }

  pub fn extend_startup(&mut self, modules: impl IntoIterator<Item = ModuleIdentifier>) {
    self.startup.extend(modules);
  }
}

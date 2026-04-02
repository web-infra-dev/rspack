use rspack_collections::IdentifierMap;

use crate::{
  ArtifactExt,
  incremental::{Incremental, IncrementalPasses},
};
#[cfg(test)]
use crate::ModuleIdentifier;

#[derive(Debug, Default, Clone)]
pub struct ModuleDependencyExportsAnalysis {
  dirty: bool,
}

impl ModuleDependencyExportsAnalysis {
  pub fn dirty(&self) -> bool {
    self.dirty
  }

  pub fn set_dirty(&mut self, dirty: bool) {
    self.dirty = dirty;
  }
}

#[derive(Debug, Default, Clone)]
pub struct DependencyExportsTopology;

#[derive(Debug, Default, Clone)]
pub struct DependencyExportsAnalysisArtifact {
  modules: IdentifierMap<ModuleDependencyExportsAnalysis>,
  topology: DependencyExportsTopology,
  topology_dirty: bool,
}

impl ArtifactExt for DependencyExportsAnalysisArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::FINISH_MODULES;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if incremental.mutations_readable(Self::PASS) {
      std::mem::swap(new, old);
      new.mark_all_dirty();
      new.set_topology_dirty(true);
    }
  }
}

impl DependencyExportsAnalysisArtifact {
  pub fn modules(&self) -> &IdentifierMap<ModuleDependencyExportsAnalysis> {
    &self.modules
  }

  #[cfg(test)]
  fn replace_module(
    &mut self,
    module_identifier: ModuleIdentifier,
    analysis: ModuleDependencyExportsAnalysis,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.insert(module_identifier, analysis);
    self.set_topology_dirty(true);
    previous
  }

  #[cfg(test)]
  fn remove_module(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.remove(module_identifier);
    if previous.is_some() {
      self.set_topology_dirty(true);
    }
    previous
  }

  fn mark_all_dirty(&mut self) {
    self
      .modules
      .values_mut()
      .for_each(|analysis| analysis.set_dirty(true));
  }

  pub fn topology_dirty(&self) -> bool {
    self.topology_dirty
  }

  pub fn topology(&self) -> &DependencyExportsTopology {
    &self.topology
  }

  fn set_topology_dirty(&mut self, topology_dirty: bool) {
    self.topology_dirty = topology_dirty;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    ModuleIdentifier,
    incremental::{Incremental, IncrementalOptions, IncrementalPasses},
  };

  #[test]
  fn recover_keeps_previous_finish_modules_state_and_marks_it_dirty() {
    let module = ModuleIdentifier::from("module-a");
    let mut old = DependencyExportsAnalysisArtifact::default();
    old.replace_module(module, ModuleDependencyExportsAnalysis::default());
    old.set_topology_dirty(false);

    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::FINISH_MODULES,
    });

    let mut new = DependencyExportsAnalysisArtifact::default();
    DependencyExportsAnalysisArtifact::recover(&incremental, &mut new, &mut old);

    assert!(new.modules().contains_key(&module));
    assert!(
      new
        .modules()
        .get(&module)
        .expect("module should recover")
        .dirty()
    );
    assert!(new.topology_dirty());
  }

  #[test]
  fn module_mutations_mark_topology_dirty() {
    let module = ModuleIdentifier::from("module-a");
    let mut artifact = DependencyExportsAnalysisArtifact::default();

    artifact.set_topology_dirty(false);
    artifact.replace_module(module, ModuleDependencyExportsAnalysis::default());
    assert!(artifact.topology_dirty());

    artifact.set_topology_dirty(false);
    artifact.remove_module(&module);
    assert!(artifact.topology_dirty());
  }
}

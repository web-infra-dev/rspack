use rspack_collections::IdentifierMap;

use crate::{
  ArtifactExt, ModuleIdentifier,
  incremental::{Incremental, IncrementalPasses},
};

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

  pub fn modules_mut(&mut self) -> &mut IdentifierMap<ModuleDependencyExportsAnalysis> {
    &mut self.modules
  }

  pub fn replace_module(
    &mut self,
    module_identifier: ModuleIdentifier,
    analysis: ModuleDependencyExportsAnalysis,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    self.modules.insert(module_identifier, analysis)
  }

  pub fn remove_module(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    self.modules.remove(module_identifier)
  }

  pub fn mark_all_dirty(&mut self) {
    self
      .modules
      .values_mut()
      .for_each(|analysis| analysis.set_dirty(true));
  }

  pub fn topology(&self) -> &DependencyExportsTopology {
    &self.topology
  }

  pub fn topology_mut(&mut self) -> &mut DependencyExportsTopology {
    &mut self.topology
  }

  pub fn topology_dirty(&self) -> bool {
    self.topology_dirty
  }

  pub fn set_topology_dirty(&mut self, topology_dirty: bool) {
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
}

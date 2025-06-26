use rspack_collections::IdentifierSet;
use rspack_error::Diagnostic;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  utils::FileCounter, BuildDependency, DependencyId, FactorizeInfo, ModuleGraph,
  ModuleGraphPartial, ModuleIdentifier,
};

#[derive(Debug)]
pub enum MakeArtifactState {
  Uninitialized(HashSet<DependencyId>),
  Initialized,
}

impl Default for MakeArtifactState {
  fn default() -> Self {
    MakeArtifactState::Uninitialized(Default::default())
  }
}

#[derive(Debug, Default)]
pub struct MakeArtifact {
  // temporary data, used by subsequent steps of make
  // should be reset when rebuild
  pub built_modules: IdentifierSet,
  pub revoked_modules: IdentifierSet,
  // Field to mark whether artifact has been initialized.
  // Only Default::default() is Uninitialized, `update_module_graph` will set this field to Initialized
  // Persistent cache will update MakeArtifact and set force_build_deps to this field when this is Uninitialized.
  pub state: MakeArtifactState,

  // data
  pub module_graph_partial: ModuleGraphPartial,
  // statistical data, which can be regenerated from module_graph_partial and used as index.
  pub make_failed_module: IdentifierSet,
  pub make_failed_dependencies: HashSet<DependencyId>,
  pub entry_dependencies: HashSet<DependencyId>,
  pub file_dependencies: FileCounter,
  pub context_dependencies: FileCounter,
  pub missing_dependencies: FileCounter,
  pub build_dependencies: FileCounter,
}

impl MakeArtifact {
  pub fn get_module_graph(&self) -> ModuleGraph {
    ModuleGraph::new([Some(&self.module_graph_partial), None], None)
  }
  pub fn get_module_graph_mut(&mut self) -> ModuleGraph {
    ModuleGraph::new([None, None], Some(&mut self.module_graph_partial))
  }
  // TODO remove it
  pub fn get_module_graph_partial(&self) -> &ModuleGraphPartial {
    &self.module_graph_partial
  }
  // TODO remove it
  pub fn get_module_graph_partial_mut(&mut self) -> &mut ModuleGraphPartial {
    &mut self.module_graph_partial
  }

  pub fn revoke_module(&mut self, module_identifier: &ModuleIdentifier) -> Vec<BuildDependency> {
    let mut mg = ModuleGraph::new([None, None], Some(&mut self.module_graph_partial));
    let module = mg
      .module_by_identifier(module_identifier)
      .expect("should have module");
    // clean module build info
    let build_info = module.build_info();
    self
      .file_dependencies
      .remove_batch_file(&build_info.file_dependencies);
    self
      .context_dependencies
      .remove_batch_file(&build_info.context_dependencies);
    self
      .missing_dependencies
      .remove_batch_file(&build_info.missing_dependencies);
    self
      .build_dependencies
      .remove_batch_file(&build_info.build_dependencies);
    self.make_failed_module.remove(module_identifier);

    // clean incoming & all_dependencies(outgoing) factorize info
    let mgm = mg
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    for dep_id in mgm
      .all_dependencies
      .iter()
      .chain(mgm.incoming_connections())
    {
      if !self.make_failed_dependencies.remove(dep_id) {
        continue;
      }
      // make failed dependencies clean it.
      let dep = mg.dependency_by_id(dep_id).expect("should have dependency");
      let info = FactorizeInfo::get_from(dep).expect("should have factorize info");
      self
        .file_dependencies
        .remove_batch_file(&info.file_dependencies());
      self
        .context_dependencies
        .remove_batch_file(&info.context_dependencies());
      self
        .missing_dependencies
        .remove_batch_file(&info.missing_dependencies());
    }

    self.revoked_modules.insert(*module_identifier);
    self.built_modules.remove(module_identifier);
    mg.revoke_module(module_identifier)
  }

  pub fn revoke_dependency(&mut self, dep_id: &DependencyId, force: bool) -> Vec<BuildDependency> {
    let mut mg = ModuleGraph::new([None, None], Some(&mut self.module_graph_partial));

    let revoke_dep_ids = if self.make_failed_dependencies.remove(dep_id) {
      // make failed dependencies clean it.
      let dep = mg.dependency_by_id(dep_id).expect("should have dependency");
      let info = FactorizeInfo::get_from(dep).expect("should have factorize info");
      self
        .file_dependencies
        .remove_batch_file(&info.file_dependencies());
      self
        .context_dependencies
        .remove_batch_file(&info.context_dependencies());
      self
        .missing_dependencies
        .remove_batch_file(&info.missing_dependencies());
      // related_dep_ids will contain dep_id it self
      info.related_dep_ids().into_owned()
    } else {
      vec![*dep_id]
    };
    revoke_dep_ids
      .iter()
      .filter_map(|dep_id| mg.revoke_dependency(dep_id, force))
      .collect()
  }

  pub fn reset_dependencies_incremental_info(&mut self) {
    self.file_dependencies.reset_incremental_info();
    self.context_dependencies.reset_incremental_info();
    self.missing_dependencies.reset_incremental_info();
    self.build_dependencies.reset_incremental_info();
  }

  pub fn diagnostics(&self) -> Vec<Diagnostic> {
    let mg = self.get_module_graph();
    let module_diagnostics = self
      .make_failed_module
      .iter()
      .flat_map(|module_identifier| {
        let m = mg
          .module_by_identifier(module_identifier)
          .expect("should have module");
        m.diagnostics()
          .iter()
          .cloned()
          .map(|d| d.with_module_identifier(Some(*module_identifier)))
          .collect::<Vec<_>>()
      });
    let dep_diagnostics = self.make_failed_dependencies.iter().flat_map(|dep_id| {
      let dep = mg.dependency_by_id(dep_id).expect("should have dependency");
      let origin_module_identifier = mg.get_parent_module(dep_id);
      FactorizeInfo::get_from(dep)
        .expect("should have factorize info")
        .diagnostics()
        .iter()
        .cloned()
        .map(|d| d.with_module_identifier(origin_module_identifier.copied()))
        .collect::<Vec<_>>()
    });
    module_diagnostics.chain(dep_diagnostics).collect()
  }

  pub fn reset_temporary_data(&mut self) {
    self.built_modules = Default::default();
    self.revoked_modules = Default::default();
  }
}

use std::hash::BuildHasherDefault;

use rspack_collections::{IdentifierHasher, IdentifierSet};
use rspack_error::Diagnostic;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  BuildDependency, DependencyId, FactorizeInfo, ModuleGraph, ModuleGraphPartial, ModuleIdentifier,
  compilation::build_module_graph::ModuleToLazyMake,
  incremental_info::IncrementalInfo,
  utils::{FileCounter, ResourceId},
};

/// Enum used to mark whether module graph has been built.
///
/// The persistent cache will recovery `MakeArtifact` when `MakeArtifact.state` is `Uninitialized`.
/// Make stage will update `MakeArtifact.state` to `Initialized`, and incremental rebuild will reuse
/// the previous MakeArtifact, so persistent cache will never recovery again.
#[derive(Debug, Default)]
pub enum BuildModuleGraphArtifactState {
  #[default]
  Uninitialized,
  Initialized,
}

/// Make Artifact, including all side effects of the make stage.
#[derive(Debug, Default)]
pub struct BuildModuleGraphArtifact {
  // temporary data, used by subsequent steps of BuildModuleGraph, should be reset when rebuild.
  /// BuildModuleGraph stage affected modules.
  ///
  /// This field will contain added modules, updated modules, removed modules.
  pub affected_modules: IncrementalInfo<ModuleIdentifier, BuildHasherDefault<IdentifierHasher>>,
  /// BuildModuleGraph stage affected dependencies.
  ///
  /// This field will contain added dependencies, updated dependencies, removed dependencies.
  pub affected_dependencies: IncrementalInfo<DependencyId>,
  /// The modules which mgm.issuer() has been updated in cutout::fix_issuers.
  ///
  /// This field is empty on a cold start.
  pub issuer_update_modules: IdentifierSet,

  // data
  /// Field to mark whether artifact has been initialized.
  ///
  /// Only Default::default() is Uninitialized, `update_module_graph` will set this field to Initialized
  /// Persistent cache will update BuildModuleGraphArtifact and set force_build_deps to this field when this is Uninitialized.
  pub state: BuildModuleGraphArtifactState,
  /// Module graph data
  pub module_graph: ModuleGraph,
  pub module_to_lazy_make: ModuleToLazyMake,

  // statistical data, which can be regenerated from module_graph_partial and used as index.
  /// Diagnostic non-empty modules in the module graph.
  pub make_failed_module: IdentifierSet,
  /// Factorize failed dependencies in module graph
  pub make_failed_dependencies: HashSet<DependencyId>,
  /// Entry dependencies in the module graph
  pub entry_dependencies: HashSet<DependencyId>,
  /// The files that current module graph depends on.
  pub file_dependencies: FileCounter,
  /// The directory that current module graph depends on.
  pub context_dependencies: FileCounter,
  /// The missing files that current module graph depends on.
  pub missing_dependencies: FileCounter,
  /// The files which cache depends on.
  pub build_dependencies: FileCounter,
}

impl BuildModuleGraphArtifact {
  pub fn get_module_graph(&self) -> &ModuleGraph {
    &self.module_graph
  }
  pub fn get_module_graph_mut(&mut self) -> &mut ModuleGraph {
    &mut self.module_graph
  }
  // TODO remove it
  pub fn get_module_graph_partial(&self) -> &ModuleGraphPartial {
    &self.module_graph
  }
  // TODO remove it
  pub fn get_module_graph_partial_mut(&mut self) -> &mut ModuleGraphPartial {
    &mut self.module_graph
  }

  /// revoke a module and return multiple parent ModuleIdentifier and DependencyId pair that can generate it.
  ///
  /// This function will update index on MakeArtifact.
  pub fn revoke_module(&mut self, module_identifier: &ModuleIdentifier) -> Vec<BuildDependency> {
    let mg = &mut self.module_graph;
    let module = mg
      .module_by_identifier(module_identifier)
      .expect("should have module");
    // clean module build info
    let build_info = module.build_info();
    let resource_id = ResourceId::from(module_identifier);
    self
      .file_dependencies
      .remove_files(&resource_id, &build_info.file_dependencies);
    self
      .context_dependencies
      .remove_files(&resource_id, &build_info.context_dependencies);
    self
      .missing_dependencies
      .remove_files(&resource_id, &build_info.missing_dependencies);
    self
      .build_dependencies
      .remove_files(&resource_id, &build_info.build_dependencies);
    self.make_failed_module.remove(module_identifier);

    // clean incoming & all_dependencies(outgoing) factorize info
    let mgm = mg
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    for dep_id in mgm
      .all_dependencies
      .clone()
      .into_iter()
      .chain(mgm.incoming_connections().clone())
    {
      self.make_failed_dependencies.remove(&dep_id);

      let dep = mg
        .dependency_by_id_mut(&dep_id)
        .expect("should have dependency");
      if let Some(info) = FactorizeInfo::revoke(dep) {
        let resource_id = ResourceId::from(dep_id);
        self
          .file_dependencies
          .remove_files(&resource_id, info.file_dependencies());
        self
          .context_dependencies
          .remove_files(&resource_id, info.context_dependencies());
        self
          .missing_dependencies
          .remove_files(&resource_id, info.missing_dependencies());
      }
      self.affected_dependencies.mark_as_remove(&dep_id);
    }

    self.affected_modules.mark_as_remove(module_identifier);
    self.issuer_update_modules.remove(module_identifier);
    mg.revoke_module(module_identifier)
  }

  /// revoke a dependency and return parent ModuleIdentifier and itself pair.
  ///
  /// If `force` is true, the dependency will be completely removed, and nothing will be returned.
  /// This function will update index on MakeArtifact.
  pub fn revoke_dependency(&mut self, dep_id: &DependencyId, force: bool) -> Vec<BuildDependency> {
    self.make_failed_dependencies.remove(dep_id);

    let mg = &mut self.module_graph;
    let revoke_dep_ids = if let Some(factorize_info) = mg
      .dependency_by_id_mut(dep_id)
      .and_then(FactorizeInfo::revoke)
    {
      let resource_id = ResourceId::from(dep_id);
      self
        .file_dependencies
        .remove_files(&resource_id, factorize_info.file_dependencies());
      self
        .context_dependencies
        .remove_files(&resource_id, factorize_info.context_dependencies());
      self
        .missing_dependencies
        .remove_files(&resource_id, factorize_info.missing_dependencies());
      // related_dep_ids will contain dep_id it self
      factorize_info.related_dep_ids().to_vec()
    } else {
      vec![*dep_id]
    };
    revoke_dep_ids
      .iter()
      .filter_map(|dep_id| {
        self.affected_dependencies.mark_as_remove(dep_id);
        mg.revoke_dependency(dep_id, force)
      })
      .collect()
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
          .map(|mut d| {
            d.module_identifier = Some(*module_identifier);
            d
          })
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
        .map(|mut d| {
          d.module_identifier = origin_module_identifier.copied();
          d
        })
        .collect::<Vec<_>>()
    });
    module_diagnostics.chain(dep_diagnostics).collect()
  }

  pub fn reset_temporary_data(&mut self) {
    self.affected_modules.reset();
    self.affected_dependencies.reset();

    self.file_dependencies.reset_incremental_info();
    self.context_dependencies.reset_incremental_info();
    self.missing_dependencies.reset_incremental_info();
    self.build_dependencies.reset_incremental_info();
  }

  pub fn built_modules(&self) -> impl Iterator<Item = &ModuleIdentifier> {
    self.affected_modules.active()
  }
  pub fn revoked_modules(&self) -> impl Iterator<Item = &ModuleIdentifier> {
    self.affected_modules.dirty()
  }
}

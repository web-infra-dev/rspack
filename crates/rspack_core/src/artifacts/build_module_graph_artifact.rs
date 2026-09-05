use std::hash::BuildHasherDefault;

use rspack_collections::{IdentifierHasher, IdentifierSet};
use rspack_error::Diagnostic;
use rustc_hash::FxHashSet;

use crate::{
  ArtifactExt, BuildDependency, BuildResult, DependencyId, DependencyParents, DependencyRef,
  FactorizationArtifact, FactorizeInfo, ModuleGraph, ModuleIdentifier, SideEffectsStateArtifact,
  compilation::build_module_graph::{LazyDependencies, ModuleToLazyMake},
  incremental::IncrementalPasses,
  incremental_info::IncrementalInfo,
  module_graph::ModuleBuildData,
  utils::{FileCounter, ResourceId},
};

/// Make Artifact, including all side effects of the make stage.
#[derive(Debug)]
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
  /// This field is empty on the initial compilation.
  pub issuer_update_modules: IdentifierSet,

  // data
  /// Module graph data
  pub module_graph: ModuleGraph,
  /// Factorization results and invalidation metadata, grouped by dependency.
  pub(crate) factorization_artifact: FactorizationArtifact,
  pub side_effects_state_artifact: SideEffectsStateArtifact,
  pub module_to_lazy_make: ModuleToLazyMake,

  // statistical data, which can be regenerated from module_graph and factorization_artifact.
  /// Diagnostic non-empty modules in the module graph.
  pub make_failed_module: IdentifierSet,
  /// Factorize failed dependencies in module graph
  pub make_failed_dependencies: FxHashSet<DependencyId>,
  /// Entry dependencies in the module graph
  pub entry_dependencies: FxHashSet<DependencyId>,
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
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      affected_modules: Default::default(),
      affected_dependencies: Default::default(),
      issuer_update_modules: Default::default(),
      module_graph: Default::default(),
      factorization_artifact: Default::default(),
      side_effects_state_artifact: Default::default(),
      module_to_lazy_make: Default::default(),
      make_failed_module: Default::default(),
      make_failed_dependencies: Default::default(),
      entry_dependencies: Default::default(),
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
    }
  }

  pub fn get_module_graph(&self) -> &ModuleGraph {
    &self.module_graph
  }
  pub fn get_module_graph_mut(&mut self) -> &mut ModuleGraph {
    &mut self.module_graph
  }

  /// Installs fresh and cached builds through the same graph and index updates.
  /// The module record retains the build output after its indices are published.
  pub(crate) fn apply_build_result(
    &mut self,
    result: BuildResult,
  ) -> (ModuleIdentifier, Vec<DependencyId>, LazyDependencies) {
    let BuildResult {
      mut module,
      dependencies,
      blocks,
      optimization_bailouts,
    } = result;
    let identifier = module.identifier();
    let mut data = ModuleBuildData {
      dependencies,
      blocks,
      optimization_bailouts,
    };
    data.normalize_blocks();

    if !module.diagnostics().is_empty() {
      self.make_failed_module.insert(identifier);
    }
    let build_info = module.build_info();
    let resource = ResourceId::from(identifier);
    self
      .file_dependencies
      .add_files(&resource, &build_info.dependencies.file);
    self
      .context_dependencies
      .add_files(&resource, &build_info.dependencies.context);
    self
      .missing_dependencies
      .add_files(&resource, &build_info.dependencies.missing);
    self
      .build_dependencies
      .add_files(&resource, &build_info.dependencies.build);

    let mut all_dependencies = Vec::new();
    let mut lazy_dependencies = LazyDependencies::default();
    for (block, dependencies) in std::iter::once((None, data.dependencies.as_slice())).chain(
      data
        .blocks
        .iter()
        .map(|block| (Some(block.identifier()), block.dependency_refs())),
    ) {
      for (index_in_block, dependency) in dependencies.iter().enumerate() {
        let id = *dependency.id();
        if let Some(until) = dependency.lazy() {
          lazy_dependencies.insert(dependency, until);
        }
        if block.is_none() {
          module.add_dependency_id(id);
        }
        all_dependencies.push(id);
        self.module_graph.set_parents(
          id,
          DependencyParents {
            block,
            module: identifier,
            index_in_block,
          },
        );
        self.module_graph.add_dependency_ref(dependency.clone());
      }
    }
    for block in &data.blocks {
      module.add_block_id(block.identifier());
    }
    let mgm = self
      .module_graph
      .module_graph_module_by_identifier_mut(&identifier);
    mgm.all_dependencies_mut().clone_from(&all_dependencies);
    mgm
      .optimization_bailout_mut()
      .extend(data.optimization_bailouts.iter().cloned());
    self.module_graph.add_module_with_build_data(module, data);
    (identifier, all_dependencies, lazy_dependencies)
  }

  /// Add a dependency that has not been factorized in the current build.
  ///
  /// Replacing a dependency with the same id must also discard the previous
  /// dependency's factorization result.
  pub(crate) fn add_unfactorized_dependency(&mut self, dependency: DependencyRef) {
    self.revoke_factorization(dependency.id());
    self.module_graph.add_dependency_ref(dependency);
  }

  pub fn factorize_info(&self, dep_id: &DependencyId) -> Option<&FactorizeInfo> {
    self.factorization_artifact.get(dep_id)
  }

  pub(crate) fn record_factorization(&mut self, factorize_info: FactorizeInfo) {
    let owner_dep_id = factorize_info.owner_dep_id();
    self.revoke_factorization(&owner_dep_id);

    if !factorize_info.is_success() {
      self.make_failed_dependencies.insert(owner_dep_id);
    }

    let resource_id = ResourceId::from(owner_dep_id);
    self
      .file_dependencies
      .add_files(&resource_id, factorize_info.file_dependencies());
    self
      .context_dependencies
      .add_files(&resource_id, factorize_info.context_dependencies());
    self
      .missing_dependencies
      .add_files(&resource_id, factorize_info.missing_dependencies());

    self.factorization_artifact.insert(factorize_info);
  }

  fn revoke_factorization(&mut self, dep_id: &DependencyId) -> Option<Vec<DependencyId>> {
    let (owner_dep_id, factorize_info) = self.factorization_artifact.revoke(dep_id)?;
    self.make_failed_dependencies.remove(&owner_dep_id);

    let resource_id = ResourceId::from(owner_dep_id);
    self
      .file_dependencies
      .remove_files(&resource_id, factorize_info.file_dependencies());
    self
      .context_dependencies
      .remove_files(&resource_id, factorize_info.context_dependencies());
    self
      .missing_dependencies
      .remove_files(&resource_id, factorize_info.missing_dependencies());

    Some(factorize_info.related_dep_ids().to_vec())
  }

  pub fn steal_side_effects_state_artifact(&mut self) -> SideEffectsStateArtifact {
    std::mem::take(&mut self.side_effects_state_artifact)
  }

  pub fn set_side_effects_state_artifact(
    &mut self,
    side_effects_state_artifact: SideEffectsStateArtifact,
  ) {
    self.side_effects_state_artifact = side_effects_state_artifact;
  }

  /// revoke a module and return multiple parent ModuleIdentifier and DependencyId pair that can generate it.
  ///
  /// This function will update index on MakeArtifact.
  pub fn revoke_module(&mut self, module_identifier: &ModuleIdentifier) -> Vec<BuildDependency> {
    let module = self
      .module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");
    // clean module build info
    let build_info = module.build_info();
    let resource_id = ResourceId::from(module_identifier);
    self
      .file_dependencies
      .remove_files(&resource_id, &build_info.dependencies.file);
    self
      .context_dependencies
      .remove_files(&resource_id, &build_info.dependencies.context);
    self
      .missing_dependencies
      .remove_files(&resource_id, &build_info.dependencies.missing);
    self
      .build_dependencies
      .remove_files(&resource_id, &build_info.dependencies.build);
    self.make_failed_module.remove(module_identifier);

    // clean incoming & all_dependencies(outgoing) factorize info
    let mgm = self
      .module_graph
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have mgm");
    let dep_ids = mgm
      .all_dependencies()
      .iter()
      .copied()
      .chain(mgm.incoming_connections().clone())
      .collect::<Vec<_>>();
    for dep_id in dep_ids {
      self.make_failed_dependencies.remove(&dep_id);
      self.revoke_factorization(&dep_id);
      self.affected_dependencies.mark_as_remove(&dep_id);
    }

    self.affected_modules.mark_as_remove(module_identifier);
    self.issuer_update_modules.remove(module_identifier);
    self.module_graph.revoke_module(module_identifier)
  }

  /// revoke a dependency and return parent ModuleIdentifier and itself pair.
  ///
  /// If `force` is true, the dependency will be completely removed, and nothing will be returned.
  /// This function will update index on MakeArtifact.
  pub fn revoke_dependency(&mut self, dep_id: &DependencyId, force: bool) -> Vec<BuildDependency> {
    self.make_failed_dependencies.remove(dep_id);

    let revoke_dep_ids = self
      .revoke_factorization(dep_id)
      .unwrap_or_else(|| vec![*dep_id]);
    let mg = &mut self.module_graph;
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
      let origin_module_identifier = mg.get_parent_module(dep_id);
      self
        .factorize_info(dep_id)
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
    self.issuer_update_modules.clear();
    self.side_effects_state_artifact = Default::default();

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

impl ArtifactExt for BuildModuleGraphArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::BUILD_MODULE_GRAPH;
  fn recover(incremental: &crate::incremental::Incremental, new: &mut Self, old: &mut Self) {
    if incremental.mutations_readable(Self::PASS) {
      std::mem::swap(new, old);
      new.get_module_graph_mut().reset();
      new.side_effects_state_artifact = Default::default();
    }
  }
}

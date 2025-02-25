pub mod cutout;
pub mod repair;

use rspack_collections::IdentifierSet;
use rspack_error::{Diagnostic, Result};
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use self::{cutout::Cutout, repair::repair};
use crate::{
  utils::FileCounter, BuildDependency, Compilation, DependencyId, FactorizeInfo, ModuleGraph,
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
    ModuleGraph::new(vec![&self.module_graph_partial], None)
  }
  pub fn get_module_graph_mut(&mut self) -> ModuleGraph {
    ModuleGraph::new(vec![], Some(&mut self.module_graph_partial))
  }
  // TODO remove it
  pub fn get_module_graph_partial(&self) -> &ModuleGraphPartial {
    &self.module_graph_partial
  }
  // TODO remove it
  pub fn get_module_graph_partial_mut(&mut self) -> &mut ModuleGraphPartial {
    &mut self.module_graph_partial
  }

  fn revoke_module(&mut self, module_identifier: &ModuleIdentifier) -> Vec<BuildDependency> {
    let mut mg = ModuleGraph::new(vec![], Some(&mut self.module_graph_partial));
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
    mg.revoke_module(module_identifier)
  }

  pub fn revoke_dependency(&mut self, dep_id: &DependencyId, force: bool) -> Vec<BuildDependency> {
    let mut mg = ModuleGraph::new(vec![], Some(&mut self.module_graph_partial));

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
}

#[derive(Debug, Clone)]
pub enum MakeParam {
  BuildEntry(HashSet<DependencyId>),
  BuildEntryAndClean(HashSet<DependencyId>),
  CheckNeedBuild,
  ModifiedFiles(HashSet<ArcPath>),
  RemovedFiles(HashSet<ArcPath>),
  ForceBuildDeps(HashSet<DependencyId>),
  ForceBuildModules(IdentifierSet),
}

pub async fn make_module_graph(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
) -> Result<MakeArtifact> {
  let mut params = Vec::with_capacity(6);

  if !compilation.entries.is_empty() {
    params.push(MakeParam::BuildEntry(
      compilation
        .entries
        .values()
        .flat_map(|item| item.all_dependencies())
        .chain(compilation.global_entry.all_dependencies())
        .copied()
        .collect(),
    ));
  }
  params.push(MakeParam::CheckNeedBuild);
  if !compilation.modified_files.is_empty() {
    params.push(MakeParam::ModifiedFiles(compilation.modified_files.clone()));
  }
  if !compilation.removed_files.is_empty() {
    params.push(MakeParam::RemovedFiles(compilation.removed_files.clone()));
  }
  if let MakeArtifactState::Uninitialized(force_build_deps) = &artifact.state {
    params.push(MakeParam::ForceBuildDeps(force_build_deps.clone()));
  }

  // reset temporary data
  artifact.built_modules = Default::default();
  artifact.revoked_modules = Default::default();
  artifact = update_module_graph(compilation, artifact, params).await?;
  Ok(artifact)
}

pub async fn update_module_graph(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  params: Vec<MakeParam>,
) -> Result<MakeArtifact> {
  artifact.state = MakeArtifactState::Initialized;
  let mut cutout = Cutout::default();
  let build_dependencies = cutout.cutout_artifact(&mut artifact, params);

  compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(&artifact.revoked_modules)
    .await?;

  artifact = repair(compilation, artifact, build_dependencies).await?;
  cutout.fix_artifact(&mut artifact);
  Ok(artifact)
}

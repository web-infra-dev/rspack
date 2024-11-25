mod cutout;
pub mod repair;

use std::path::PathBuf;

use rspack_collections::IdentifierSet;
use rspack_error::{Diagnostic, Result};
use rustc_hash::FxHashSet as HashSet;

use self::{cutout::Cutout, repair::repair};
use crate::{
  utils::FileCounter, BuildDependency, Compilation, DependencyId, ModuleGraph, ModuleGraphPartial,
  ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct MakeArtifact {
  // temporary data, used by subsequent steps of make
  // should be reset when rebuild
  pub diagnostics: Vec<Diagnostic>,
  pub has_module_graph_change: bool,

  // data
  pub built_modules: IdentifierSet,
  pub revoked_modules: IdentifierSet,
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: IdentifierSet,
  pub module_graph_partial: ModuleGraphPartial,
  entry_dependencies: HashSet<DependencyId>,
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

  pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
    std::mem::take(&mut self.diagnostics)
  }

  pub fn take_built_modules(&mut self) -> IdentifierSet {
    std::mem::take(&mut self.built_modules)
  }

  pub fn take_revoked_modules(&mut self) -> IdentifierSet {
    std::mem::take(&mut self.revoked_modules)
  }

  fn revoke_module(&mut self, module_identifier: &ModuleIdentifier) -> Vec<BuildDependency> {
    let mut module_graph = ModuleGraph::new(vec![], Some(&mut self.module_graph_partial));
    let module = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");
    if let Some(build_info) = module.build_info() {
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
    }
    self.revoked_modules.insert(*module_identifier);
    module_graph.revoke_module(module_identifier)
  }

  pub fn reset_dependencies_incremental_info(&mut self) {
    self.file_dependencies.reset_incremental_info();
    self.context_dependencies.reset_incremental_info();
    self.missing_dependencies.reset_incremental_info();
    self.build_dependencies.reset_incremental_info();
  }
}

#[derive(Debug, Clone)]
pub enum MakeParam {
  BuildEntry(HashSet<DependencyId>),
  BuildEntryAndClean(HashSet<DependencyId>),
  CheckNeedBuild,
  ModifiedFiles(HashSet<PathBuf>),
  RemovedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
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
  if !artifact.make_failed_module.is_empty() {
    let make_failed_module = std::mem::take(&mut artifact.make_failed_module);
    params.push(MakeParam::ForceBuildModules(make_failed_module));
  }
  if !artifact.make_failed_dependencies.is_empty() {
    let make_failed_dependencies = std::mem::take(&mut artifact.make_failed_dependencies);
    params.push(MakeParam::ForceBuildDeps(make_failed_dependencies));
  }

  // reset temporary data
  artifact.built_modules = Default::default();
  artifact.revoked_modules = Default::default();
  artifact.diagnostics = Default::default();
  artifact.has_module_graph_change = false;

  artifact = update_module_graph(compilation, artifact, params).await?;
  Ok(artifact)
}

pub async fn update_module_graph(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  params: Vec<MakeParam>,
) -> Result<MakeArtifact> {
  let mut cutout = Cutout::default();
  let build_dependencies = cutout.cutout_artifact(&mut artifact, params);
  artifact = repair(compilation, artifact, build_dependencies).await?;
  cutout.fix_artifact(&mut artifact);
  Ok(artifact)
}

mod cutout;
pub mod repair;

use std::{hash::BuildHasherDefault, path::PathBuf};

use indexmap::IndexSet;
use rayon::prelude::*;
use rspack_error::{Diagnostic, Result};
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rustc_hash::{FxHashSet as HashSet, FxHasher};

use self::{cutout::Cutout, repair::repair};
use crate::{
  tree_shaking::{visitor::OptimizeAnalyzeResult, BailoutFlag},
  BuildDependency, Compilation, DependencyId, DependencyType, ModuleGraph, ModuleGraphPartial,
  ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct MakeArtifact {
  pub module_graph_partial: ModuleGraphPartial,
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: HashSet<ModuleIdentifier>,
  pub diagnostics: Vec<Diagnostic>,

  entry_dependencies: HashSet<DependencyId>,
  entry_module_identifiers: IdentifierSet,
  pub optimize_analyze_result_map: IdentifierMap<OptimizeAnalyzeResult>,
  pub file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,

  pub has_module_graph_change: bool,
}

impl MakeArtifact {
  fn get_module_graph(&self) -> ModuleGraph {
    ModuleGraph::new(vec![&self.module_graph_partial], None)
  }
  fn get_module_graph_mut(&mut self) -> ModuleGraph {
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

  // TODO remove it
  fn move_data_from_compilation(&mut self, compilation: &mut Compilation) {
    self.entry_module_identifiers = std::mem::take(&mut compilation.entry_module_identifiers);
    self.file_dependencies = std::mem::take(&mut compilation.file_dependencies);
    self.context_dependencies = std::mem::take(&mut compilation.context_dependencies);
    self.missing_dependencies = std::mem::take(&mut compilation.missing_dependencies);
    self.build_dependencies = std::mem::take(&mut compilation.build_dependencies);
  }

  // TODO remove it
  fn move_data_to_compilation(&mut self, compilation: &mut Compilation) {
    compilation.entry_module_identifiers = std::mem::take(&mut self.entry_module_identifiers);
    compilation.file_dependencies = std::mem::take(&mut self.file_dependencies);
    compilation.context_dependencies = std::mem::take(&mut self.context_dependencies);
    compilation.missing_dependencies = std::mem::take(&mut self.missing_dependencies);
    compilation.build_dependencies = std::mem::take(&mut self.build_dependencies);

    compilation.push_batch_diagnostic(std::mem::take(&mut self.diagnostics));
  }
}

#[derive(Debug, Clone)]
pub enum MakeParam {
  Entry(HashSet<DependencyId>),
  ModifiedFiles(HashSet<PathBuf>),
  RemovedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

pub fn make_module_graph(
  compilation: &mut Compilation,
  mut artifact: MakeArtifact,
) -> Result<MakeArtifact> {
  let mut params = Vec::with_capacity(5);

  if !compilation.entries.is_empty() {
    params.push(MakeParam::Entry(
      compilation
        .entries
        .values()
        .flat_map(|item| &item.dependencies)
        .chain(&compilation.global_entry.dependencies)
        .cloned()
        .collect(),
    ));
  }
  // no modified files but rebuild means force build
  // some module which cacheable is false will need to be rebuilt even if modified files is empty
  params.push(MakeParam::ModifiedFiles(compilation.modified_files.clone()));
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

  // reset diagnostics
  artifact.diagnostics = Default::default();
  artifact.has_module_graph_change = false;

  artifact.move_data_from_compilation(compilation);

  artifact = update_module_graph_with_artifact(compilation, artifact, params)?;

  if compilation.options.builtins.tree_shaking.enable() {
    let module_graph = artifact.get_module_graph();
    compilation.bailout_module_identifiers = calc_bailout_module_identifiers(&module_graph);
  }

  artifact.move_data_to_compilation(compilation);
  Ok(artifact)
}

pub async fn update_module_graph(
  compilation: &mut Compilation,
  params: Vec<MakeParam>,
) -> Result<()> {
  let mut artifact = MakeArtifact::default();
  compilation.swap_make_artifact(&mut artifact);
  artifact.move_data_from_compilation(compilation);

  artifact = update_module_graph_with_artifact(compilation, artifact, params)?;

  if compilation.options.builtins.tree_shaking.enable() {
    let module_graph = artifact.get_module_graph();
    compilation.bailout_module_identifiers = calc_bailout_module_identifiers(&module_graph);
  }

  artifact.move_data_to_compilation(compilation);
  compilation.swap_make_artifact(&mut artifact);
  Ok(())
}

pub fn update_module_graph_with_artifact(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  params: Vec<MakeParam>,
) -> Result<MakeArtifact> {
  let mut cutout = Cutout::default();
  let build_dependencies = cutout.cutout_artifact(&mut artifact, params);
  artifact = repair(compilation, artifact, build_dependencies)?;
  cutout.fix_artifact(&mut artifact);

  Ok(artifact)
}

fn calc_bailout_module_identifiers(module_graph: &ModuleGraph) -> IdentifierMap<BailoutFlag> {
  // Avoid to introduce too much overhead,
  // until we find a better way to align with webpack hmr behavior

  // add context module and context element module to bailout_module_identifiers
  module_graph
    .dependencies()
    .values()
    .par_bridge()
    .filter_map(|dep| {
      if dep.as_context_dependency().is_some()
        && let Some(module) = module_graph.get_module_by_dependency_id(dep.id())
      {
        let mut values = vec![(module.identifier(), BailoutFlag::CONTEXT_MODULE)];
        if let Some(dependencies) = module_graph.get_module_all_dependencies(&module.identifier()) {
          for dependency in dependencies {
            if let Some(dependency_module) =
              module_graph.module_identifier_by_dependency_id(dependency)
            {
              values.push((*dependency_module, BailoutFlag::CONTEXT_MODULE));
            }
          }
        }

        Some(values)
      } else if matches!(
        dep.dependency_type(),
        DependencyType::ContainerExposed | DependencyType::ProvideModuleForShared
      ) && let Some(module) = module_graph.get_module_by_dependency_id(dep.id())
      {
        Some(vec![(module.identifier(), BailoutFlag::CONTAINER_EXPOSED)])
      } else {
        None
      }
    })
    .flatten()
    .collect()
}

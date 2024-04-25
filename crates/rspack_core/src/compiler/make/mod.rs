mod cutout;
mod repair;

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
  module_graph_partial: ModuleGraphPartial,
  make_failed_dependencies: HashSet<BuildDependency>,
  make_failed_module: HashSet<ModuleIdentifier>,
  diagnostics: Vec<Diagnostic>,

  entry_module_identifiers: IdentifierSet,
  optimize_analyze_result_map: IdentifierMap<OptimizeAnalyzeResult>,
  file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,

  has_module_graph_change: bool,
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
    self.optimize_analyze_result_map = std::mem::take(&mut compilation.optimize_analyze_result_map);
    self.file_dependencies = std::mem::take(&mut compilation.file_dependencies);
    self.context_dependencies = std::mem::take(&mut compilation.context_dependencies);
    self.missing_dependencies = std::mem::take(&mut compilation.missing_dependencies);
    self.build_dependencies = std::mem::take(&mut compilation.build_dependencies);
    self.has_module_graph_change = compilation.has_module_import_export_change;
  }

  // TODO remove it
  fn move_data_to_compilation(&mut self, compilation: &mut Compilation) {
    compilation.entry_module_identifiers = std::mem::take(&mut self.entry_module_identifiers);
    compilation.optimize_analyze_result_map = std::mem::take(&mut self.optimize_analyze_result_map);
    compilation.file_dependencies = std::mem::take(&mut self.file_dependencies);
    compilation.context_dependencies = std::mem::take(&mut self.context_dependencies);
    compilation.missing_dependencies = std::mem::take(&mut self.missing_dependencies);
    compilation.build_dependencies = std::mem::take(&mut self.build_dependencies);
    compilation.has_module_import_export_change = self.has_module_graph_change;

    compilation.push_batch_diagnostic(std::mem::take(&mut self.diagnostics));
    compilation.make_failed_module = std::mem::take(&mut self.make_failed_module);
    compilation.make_failed_dependencies = std::mem::take(&mut self.make_failed_dependencies);
  }
}

#[derive(Debug)]
pub enum MakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  DeletedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

impl MakeParam {
  pub fn new_force_build_dep_param(dep: DependencyId, module: Option<ModuleIdentifier>) -> Self {
    let mut data = HashSet::default();
    data.insert((dep, module));
    Self::ForceBuildDeps(data)
  }
}

pub async fn update_module_graph(
  compilation: &mut Compilation,
  params: Vec<MakeParam>,
) -> Result<()> {
  let mut artifact = MakeArtifact::default();
  compilation.swap_make_artifact(&mut artifact);
  artifact.move_data_from_compilation(compilation);
  let mut cutout = Cutout::default();
  let build_dependencies = cutout.cutout_artifact(&mut artifact, params);
  artifact = repair(compilation, artifact, build_dependencies)?;
  cutout.fix_artifact(&mut artifact);

  // Avoid to introduce too much overhead,
  // until we find a better way to align with webpack hmr behavior

  // add context module and context element module to bailout_module_identifiers
  if compilation.options.builtins.tree_shaking.enable() {
    let module_graph = artifact.get_module_graph();
    compilation.bailout_module_identifiers = module_graph
      .dependencies()
      .values()
      .par_bridge()
      .filter_map(|dep| {
        if dep.as_context_dependency().is_some()
          && let Some(module) = module_graph.get_module_by_dependency_id(dep.id())
        {
          let mut values = vec![(module.identifier(), BailoutFlag::CONTEXT_MODULE)];
          if let Some(dependencies) = module_graph.get_module_all_dependencies(&module.identifier())
          {
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
      .collect();
  }

  artifact.move_data_to_compilation(compilation);
  compilation.swap_make_artifact(&mut artifact);
  Ok(())
}

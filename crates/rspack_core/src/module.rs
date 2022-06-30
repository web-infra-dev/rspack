use std::fmt::Debug;

use crate::{Compilation, Dependency, ModuleDependency, ModuleGraph, ModuleType, ResolveKind};

#[derive(Debug)]
pub struct ModuleGraphModule {
  // Only user defined entry module has name for now.
  pub name: Option<String>,
  pub id: String,
  // pub exec_order: usize,
  pub uri: String,
  pub module: BoxModule,
  pub module_type: ModuleType,
  all_dependencies: Vec<Dependency>,
}

impl ModuleGraphModule {
  pub fn new(
    name: Option<String>,
    id: String,
    uri: String,
    module: BoxModule,
    dependencies: Vec<Dependency>,
    source_type: ModuleType,
  ) -> Self {
    Self {
      name,
      id,
      // exec_order: usize::MAX,
      uri,
      module,
      all_dependencies: dependencies,
      module_type: source_type,
    }
  }

  pub fn depended_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleGraphModule> {
    self
      .all_dependencies
      .iter()
      .filter(|dep| !matches!(dep.detail.kind, ResolveKind::DynamicImport))
      .filter_map(|dep| module_graph.module_by_dependency(dep))
      .collect()
  }

  pub fn dynamic_depended_modules<'a>(
    &self,
    module_graph: &'a ModuleGraph,
  ) -> Vec<&'a ModuleGraphModule> {
    self
      .all_dependencies
      .iter()
      .filter(|dep| matches!(dep.detail.kind, ResolveKind::DynamicImport))
      .filter_map(|dep| module_graph.module_by_dependency(dep))
      .collect()
  }
}

pub trait Module: Debug + Send + Sync {
  fn render(&self, module: &ModuleGraphModule, compilation: &Compilation) -> String;

  fn dependencies(&mut self) -> Vec<ModuleDependency> {
    vec![]
  }
}

pub type BoxModule = Box<dyn Module>;

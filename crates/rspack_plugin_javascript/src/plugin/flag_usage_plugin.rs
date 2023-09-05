use std::collections::VecDeque;

use rspack_core::tree_shaking::visitor::ModuleIdOrDepId;
use rspack_core::{
  Compilation, Dependency, DependencyId, ExportsInfoId, ModuleGraph, ModuleIdentifier,
  ReferencedExport, RuntimeSpec,
};
use rustc_hash::FxHashMap as HashMap;

pub struct FlagDependencyUsagePlugin<'a> {
  global: bool,
  compilation: &'a mut Compilation,
  exports_info_module_map: HashMap<ExportsInfoId, ModuleIdentifier>,
}

impl<'a> FlagDependencyUsagePlugin<'a> {
  pub fn new(global: bool, compilation: &'a mut Compilation) -> Self {
    Self {
      global,
      compilation,
      exports_info_module_map: HashMap::default(),
    }
  }

  fn apply(&mut self) {
    for mgm in self
      .compilation
      .module_graph
      .module_graph_modules()
      .values()
    {
      self
        .exports_info_module_map
        .insert(mgm.exports, mgm.module_identifier);
    }
    let mg = &mut self.compilation.module_graph;
    for exports_info_id in self.exports_info_module_map.keys() {}
    // for (name, entry_data) in self.compilation.entries.iter() {}
  }

  fn process_entry_dependency(&mut self, dep: DependencyId, runtime: &RuntimeSpec) {
    if let Some(module) = self
      .compilation
      .module_graph
      .module_graph_module_by_dependency_id(&dep)
    {}
  }

  fn process_referenced_module(
    &mut self,
    module_id: ModuleIdentifier,
    used_exports: Vec<ReferencedExport>,
  ) {
  }
}

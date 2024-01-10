use rspack_core::{Compilation, Plugin};
use rspack_error::Result;

use crate::id_helpers::{
  assign_ascending_module_ids, compare_modules_by_pre_order_index_or_identifier,
  get_used_module_ids_and_modules,
};

#[derive(Debug, Default)]
pub struct NaturalModuleIdsPlugin;

impl Plugin for NaturalModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "rspack.NaturalModuleIdsPlugin"
  }

  fn module_ids(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = &compilation.module_graph;
    let (used_ids, module_identifiers) = get_used_module_ids_and_modules(compilation, None);
    let mut modules = module_identifiers
      .into_iter()
      .map(|id| {
        module_graph
          .module_by_identifier(&id)
          .unwrap_or_else(|| panic!("module {id} not found"))
      })
      .collect::<Vec<_>>();

    modules.sort_unstable_by(|a, b| {
      compare_modules_by_pre_order_index_or_identifier(module_graph, a, b)
    });

    let chunk_graph = &mut compilation.chunk_graph;
    assign_ascending_module_ids(&used_ids, modules, chunk_graph);
    Ok(())
  }
}

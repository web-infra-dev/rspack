use rspack_core::{
  compare_modules_by_pre_order_index_or_identifier, ApplyContext, CompilationModuleIds,
  CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{assign_ascending_module_ids, get_used_module_ids_and_modules};

#[plugin]
#[derive(Debug, Default)]
pub struct NaturalModuleIdsPlugin;

#[plugin_hook(CompilationModuleIds for NaturalModuleIdsPlugin)]
fn module_ids(&self, compilation: &mut rspack_core::Compilation) -> Result<()> {
  let (used_ids, mut modules_in_natural_order) = get_used_module_ids_and_modules(compilation, None);

  let mut chunk_graph = std::mem::take(&mut compilation.chunk_graph);
  let module_graph = compilation.get_module_graph();

  modules_in_natural_order
    .sort_unstable_by(|a, b| compare_modules_by_pre_order_index_or_identifier(&module_graph, a, b));

  let modules_in_natural_order = modules_in_natural_order
    .into_iter()
    .filter_map(|i| module_graph.module_by_identifier(&i))
    .collect::<Vec<_>>();

  assign_ascending_module_ids(&used_ids, modules_in_natural_order, &mut chunk_graph);

  compilation.chunk_graph = chunk_graph;

  Ok(())
}

impl Plugin for NaturalModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "NaturalModuleIdsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .module_ids
      .tap(module_ids::new(self));
    Ok(())
  }
}

use rspack_core::{CompilationModuleIds, Plugin, incremental::IncrementalPasses};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{
  assign_ascending_module_ids, compare_modules_by_pre_order_index_or_identifier,
  get_used_module_ids_and_modules,
};

#[plugin]
#[derive(Debug, Default)]
pub struct NaturalModuleIdsPlugin;

#[plugin_hook(CompilationModuleIds for NaturalModuleIdsPlugin)]
async fn module_ids(&self, compilation: &mut rspack_core::Compilation) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS,
    "NaturalModuleIdsPlugin (optimization.moduleIds = \"natural\")",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
    }
    compilation.module_ids_artifact.clear();
  }

  let (used_ids, mut modules_in_natural_order) = get_used_module_ids_and_modules(compilation, None);

  let mut module_ids = std::mem::take(&mut compilation.module_ids_artifact);
  let module_graph = compilation.get_module_graph();

  modules_in_natural_order
    .sort_unstable_by(|a, b| compare_modules_by_pre_order_index_or_identifier(&module_graph, a, b));

  let modules_in_natural_order = modules_in_natural_order
    .into_iter()
    .filter_map(|i| module_graph.module_by_identifier(&i))
    .collect::<Vec<_>>();

  assign_ascending_module_ids(&used_ids, modules_in_natural_order, &mut module_ids);

  compilation.module_ids_artifact = module_ids;

  Ok(())
}

impl Plugin for NaturalModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "NaturalModuleIdsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}

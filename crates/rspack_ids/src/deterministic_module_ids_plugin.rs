use rspack_core::{
  ApplyContext, ChunkGraph, Compilation, CompilationModuleIds, CompilerOptions, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{
  assign_deterministic_ids, compare_modules_by_pre_order_index_or_identifier, get_full_module_name,
  get_used_module_ids_and_modules,
};

#[plugin]
#[derive(Debug, Default)]
pub struct DeterministicModuleIdsPlugin;

#[plugin_hook(CompilationModuleIds for DeterministicModuleIdsPlugin)]
fn module_ids(&self, compilation: &mut Compilation) -> Result<()> {
  let (mut used_ids, modules) = get_used_module_ids_and_modules(compilation, None);

  let mut module_ids = std::mem::take(&mut compilation.module_ids);
  let context = compilation.options.context.as_ref();
  let max_length = 3;
  let fail_on_conflict = false;
  let fixed_length = false;
  let salt = 0;
  let mut conflicts = 0;

  let module_graph = compilation.get_module_graph();
  let modules = modules
    .into_iter()
    .filter_map(|i| module_graph.module_by_identifier(&i))
    .collect::<Vec<_>>();
  let used_ids_len = used_ids.len();
  assign_deterministic_ids(
    modules,
    |m| get_full_module_name(m, context),
    |a, b| compare_modules_by_pre_order_index_or_identifier(&module_graph, a, b),
    |module, id| {
      if !used_ids.insert(id.to_string()) {
        conflicts += 1;
        return false;
      }
      ChunkGraph::set_module_id(&mut module_ids, module.identifier(), id.to_string());
      true
    },
    &[usize::pow(10, max_length)],
    if fixed_length { 0 } else { 10 },
    used_ids_len,
    salt,
  );
  compilation.module_ids = module_ids;
  if fail_on_conflict && conflicts > 0 {
    // TODO: better error msg
    panic!("Assigning deterministic module ids has lead to conflicts {conflicts}");
  }
  Ok(())
}

impl Plugin for DeterministicModuleIdsPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .module_ids
      .tap(module_ids::new(self));
    Ok(())
  }
}

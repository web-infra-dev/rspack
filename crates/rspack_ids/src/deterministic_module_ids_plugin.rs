use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::number_hash::get_number_hash;

use crate::id_helpers::{
  compare_modules_by_pre_order_index_or_identifier, get_deterministic_id_range,
  get_full_module_name, get_used_module_ids_and_modules_with_artifact,
  precompute_deterministic_id_candidates,
};

#[plugin]
#[derive(Debug, Default)]
pub struct DeterministicModuleIdsPlugin;

#[plugin_hook(CompilationModuleIds for DeterministicModuleIdsPlugin)]
async fn module_ids(
  &self,
  compilation: &Compilation,
  module_ids: &mut ModuleIdsArtifact,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULE_IDS,
    "DeterministicModuleIdsPlugin (optimization.moduleIds = \"deterministic\")",
    "it requires calculating the id of all the modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      diagnostics.push(diagnostic);
    }
    module_ids.clear();
  }

  let (mut used_ids, modules) =
    get_used_module_ids_and_modules_with_artifact(compilation, module_ids, None);

  let mut module_ids_map = std::mem::take(module_ids);
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
  let range = get_deterministic_id_range(
    modules.len(),
    &[usize::pow(10, max_length)],
    if fixed_length { 0 } else { 10 },
    used_ids_len,
  );
  let prepared_modules = precompute_deterministic_id_candidates(
    modules,
    |m| get_full_module_name(m, context),
    |a, b| {
      compare_modules_by_pre_order_index_or_identifier(
        module_graph,
        &a.identifier(),
        &b.identifier(),
      )
    },
    range,
    salt,
  );

  for prepared in prepared_modules {
    let mut i = salt;
    let mut id = prepared.initial_id;
    while !used_ids.insert(id.to_string()) {
      conflicts += 1;
      i += 1;
      let mut i_buffer = rspack_util::itoa::Buffer::new();
      id = get_number_hash(&format!("{}{}", prepared.name, i_buffer.format(i)), range);
    }

    ChunkGraph::set_module_id(
      &mut module_ids_map,
      prepared.item.identifier(),
      id.to_string().into(),
    );
  }

  *module_ids = module_ids_map;
  if fail_on_conflict && conflicts > 0 {
    // TODO: better error msg
    panic!("Assigning deterministic module ids has lead to conflicts {conflicts}");
  }
  Ok(())
}

impl Plugin for DeterministicModuleIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}

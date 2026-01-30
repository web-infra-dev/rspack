use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rspack_core::{
  ChunkGraph, Compilation, CompilationModuleIds, ModuleIdsArtifact, Plugin,
  incremental::IncrementalPasses,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{
  assign_deterministic_ids, compare_modules_by_pre_order_index_or_identifier, get_full_module_name,
  get_used_module_ids_and_modules,
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

  let (mut used_ids, modules) = get_used_module_ids_and_modules(compilation, None);

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

  let module_names = modules
    .par_iter()
    .map(|m| (m.identifier(), get_full_module_name(m, context)))
    .collect::<IdentifierMap<String>>();

  assign_deterministic_ids(
    modules,
    |m| {
      module_names
        .get(&m.identifier())
        .expect("should have generated full module name")
        .clone()
    },
    |a, b| {
      compare_modules_by_pre_order_index_or_identifier(
        module_graph,
        &a.identifier(),
        &b.identifier(),
      )
    },
    |module, id| {
      if !used_ids.insert(id.to_string()) {
        conflicts += 1;
        return false;
      }
      ChunkGraph::set_module_id(
        &mut module_ids_map,
        module.identifier(),
        id.to_string().into(),
      );
      true
    },
    &[usize::pow(10, max_length)],
    if fixed_length { 0 } else { 10 },
    used_ids_len,
    salt,
  );
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

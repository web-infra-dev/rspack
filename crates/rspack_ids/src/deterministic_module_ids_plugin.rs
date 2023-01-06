use rspack_core::{Compilation, Plugin};
use rspack_error::Result;

use crate::id_helpers::{
  assign_deterministic_ids, compare_modules_by_pre_order_index_or_identifier, get_full_module_name,
  get_used_module_ids_and_modules,
};

#[derive(Debug)]
pub struct DeterministicModuleIdsPlugin {}

impl Plugin for DeterministicModuleIdsPlugin {
  fn module_ids(&mut self, compilation: &mut Compilation) -> Result<()> {
    let (mut used_ids, modules) = get_used_module_ids_and_modules(compilation, None);

    let module_graph = &compilation.module_graph;
    let chunk_graph = &mut compilation.chunk_graph;
    let context = &compilation.options.context.to_string_lossy();
    let max_length = 3;
    let fail_on_conflict = false;
    let fixed_length = false;
    let salt = 0;
    let mut conflicts = 0;

    let modules = modules
      .into_iter()
      .filter_map(|i| compilation.module_graph.module_by_identifier(&i))
      .collect::<Vec<_>>();
    let used_ids_len = used_ids.len();
    assign_deterministic_ids(
      modules,
      |m| get_full_module_name(m, context),
      |a, b| compare_modules_by_pre_order_index_or_identifier(module_graph, a, b),
      |module, id| {
        let size = used_ids.len();
        used_ids.insert(id.clone());
        if used_ids.len() == size {
          conflicts += 1;
          return false;
        }
        chunk_graph.set_module_id(&module.identifier(), id);
        true
      },
      &[usize::pow(10, max_length)],
      if fixed_length { 0 } else { 10 },
      used_ids_len,
      salt,
    );
    if fail_on_conflict && conflicts > 0 {
      // TODO: better error msg
      panic!("Assigning deterministic module ids has lead to conflicts {conflicts}");
    }
    Ok(())
  }
}

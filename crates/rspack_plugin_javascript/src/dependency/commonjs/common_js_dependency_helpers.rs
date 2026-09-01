use rspack_core::{Compilation, ModuleIdentifier, RuntimeSpec};

/// A classic worker entry reads top-level `this` as its global scope. Only
/// return true when this module is an entry module and every matching entry
/// role is a classic worker, so imported modules, normal entries, and module
/// worklets keep exports semantics.
pub(super) fn is_worker_entry_this(
  compilation: &Compilation,
  module: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
) -> bool {
  let Some(runtime) = runtime.filter(|runtime| runtime.len() == 1) else {
    return false;
  };
  if compilation.options.output.module {
    return false;
  }

  let artifact = &compilation.build_chunk_graph_artifact;
  let mut worker_entry = false;
  for chunk_ukey in artifact.chunk_graph.get_module_chunks(module) {
    let chunk = artifact.chunk_by_ukey.expect_get(chunk_ukey);
    if chunk.runtime() != runtime {
      continue;
    }
    let Some(group_ukey) = artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey)
      .get(&module)
    else {
      continue;
    };
    let group = artifact.chunk_group_by_ukey.expect_get(group_ukey);
    let Some(options) = group.kind.get_entry_options() else {
      return false;
    };
    if options.worker != Some(true) || options.worklet == Some(true) {
      return false;
    }
    worker_entry = true;
  }
  worker_entry
}

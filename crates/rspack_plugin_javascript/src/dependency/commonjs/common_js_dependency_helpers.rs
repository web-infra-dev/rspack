use rspack_core::{
  Compilation, DependencyId, EntryOptions, EntryRuntime, GroupOptions, ModuleGraph,
  ModuleIdentifier, RuntimeSpec,
};
use rspack_hash::{RspackHash, RspackHasher};

fn entry_options_matches_runtime(options: &EntryOptions, runtime: &RuntimeSpec) -> bool {
  let entry_runtime = match &options.runtime {
    Some(EntryRuntime::String(s)) if !s.is_empty() => Some(s.as_str()),
    _ => options.name.as_deref(),
  };
  entry_runtime.is_some_and(|name| runtime.contains(&name.into()))
}

/// Checks whether `module` is targeted by a classic browser worker entrypoint
/// block in `runtime` (or in any runtime when `runtime` is `None`, i.e. global
/// `usedExports`). Used during dependency usage analysis before chunk graph
/// construction so `this.x = require(...)` preserves the required module's
/// exports instead of treating `x` as an unused CommonJS export.
pub(super) fn is_worker_entry_module(
  mg: &ModuleGraph,
  module: ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
) -> bool {
  for conn in mg.get_incoming_connections(&module) {
    let Some(block_id) = mg.get_parent_block(&conn.dependency_id) else {
      continue;
    };
    let Some(block) = mg.block_by_id(block_id) else {
      continue;
    };
    let Some(GroupOptions::Entrypoint(options)) = block.get_group_options() else {
      continue;
    };
    if options.worker != Some(true) || options.worklet == Some(true) {
      continue;
    }
    let Some(runtime) = runtime else {
      return true;
    };
    if entry_options_matches_runtime(options, runtime) {
      return true;
    }
  }
  false
}

/// A classic worker entry reads top-level `this` as its global scope. Only
/// return true when every chunk in `runtime` containing `module` loads it as
/// an entry module of a classic browser worker entrypoint, so Node workers,
/// imported modules, normal entries, shared-runtime workers, and module
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
      return false;
    };
    let group = artifact.chunk_group_by_ukey.expect_get(group_ukey);
    let Some(options) = group.kind.get_entry_options() else {
      return false;
    };
    if options.worker != Some(true) || options.worklet == Some(true) {
      return false;
    }
    for other_group_ukey in chunk.groups() {
      let other_group = artifact.chunk_group_by_ukey.expect_get(other_group_ukey);
      if let Some(other_options) = other_group.kind.get_entry_options()
        && (other_options.worker != Some(true) || other_options.worklet == Some(true))
      {
        return false;
      }
    }
    worker_entry = true;
  }
  worker_entry
}

pub(super) fn update_worker_entry_this_hash(
  dep_id: &DependencyId,
  hasher: &mut RspackHasher,
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
) {
  let module_graph = compilation.get_module_graph();
  let worker_global = module_graph
    .get_parent_module(dep_id)
    .is_some_and(|module| is_worker_entry_this(compilation, *module, runtime));
  if worker_global {
    "worker global".hash(hasher);
  } else {
    "exports".hash(hasher);
  }
}

use rspack_core::{
  AsyncDependenciesBlock, Compilation, CompilationShouldCreateChunkGroup, DependenciesBlock,
  DependencyType,
};
use rspack_error::Result;
use rspack_hook::plugin_hook;

use crate::{JsPlugin, JsPluginInner, dependency::is_resolve_entry};

#[plugin_hook(CompilationShouldCreateChunkGroup for JsPlugin, tracing=false)]
pub(super) fn should_create_chunk_group(
  &self,
  compilation: &Compilation,
  block: &AsyncDependenciesBlock,
  create: &mut bool,
) -> Result<()> {
  let [dependency] = block.get_dependencies() else {
    return Ok(());
  };
  if dependency.dependency_type() != &DependencyType::ImportMetaResolve {
    return Ok(());
  }
  let module_graph = compilation.get_module_graph();
  if let Some(module) = module_graph
    .module_identifier_by_dependency_id(dependency.id())
    .and_then(|identifier| module_graph.module_by_identifier(identifier))
    && !is_resolve_entry(module.as_ref())
  {
    // URL exports must be synchronously available to the referencing module.
    // Only executable JavaScript targets need a standalone entry.
    *create = false;
  }
  Ok(())
}

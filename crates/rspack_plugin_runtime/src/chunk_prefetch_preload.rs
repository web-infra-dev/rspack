use rspack_core::{
  ChunkGroupOrderKey, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAdditionalTreeRuntimeRequirements, Plugin, RuntimeGlobals, RuntimeModule,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{
  ChunkPrefetchStartupRuntimeModule, ChunkPrefetchTriggerRuntimeModule,
  ChunkPreloadTriggerRuntimeModule,
};

#[plugin]
#[derive(Debug, Default)]
pub struct ChunkPrefetchPreloadPlugin;

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for ChunkPrefetchPreloadPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  if compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    == 0
  {
    return Ok(());
  }

  if let Some(startup_child_chunks) =
    chunk.get_children_of_type_in_order(&ChunkGroupOrderKey::Prefetch, compilation, false)
  {
    runtime_modules.push(Box::new(ChunkPrefetchStartupRuntimeModule::new(
      &compilation.runtime_template,
      startup_child_chunks,
    )));
  }
  Ok(())
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ChunkPrefetchPreloadPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let chunk_filter = |_: &ChunkUkey, __: &Compilation| true;
  let mut chunk_map = chunk.get_child_ids_by_orders_map(false, compilation, &chunk_filter);

  if let Some(prefetch_map) = chunk_map.remove(&ChunkGroupOrderKey::Prefetch) {
    runtime_modules.push(Box::new(ChunkPrefetchTriggerRuntimeModule::new(
      &compilation.runtime_template,
      prefetch_map,
    )));
  }

  if let Some(preload_map) = chunk_map.remove(&ChunkGroupOrderKey::Preload) {
    runtime_modules.push(Box::new(ChunkPreloadTriggerRuntimeModule::new(
      &compilation.runtime_template,
      preload_map,
    )));
  }

  Ok(())
}

impl Plugin for ChunkPrefetchPreloadPlugin {
  fn name(&self) -> &'static str {
    "ChunkPrefetchPreloadPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}

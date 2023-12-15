use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkGroupOrderKey, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeGlobals,
};

use crate::runtime_module::{
  ChunkPrefetchStartupRuntimeModule, ChunkPrefetchTriggerRuntimeModule,
  ChunkPreloadTriggerRuntimeModule,
};

#[derive(Debug)]
pub struct ChunkPrefetchPreloadPlugin;

#[async_trait]
impl Plugin for ChunkPrefetchPreloadPlugin {
  fn name(&self) -> &'static str {
    "ChunkPrefetchPreloadPlugin"
  }

  fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let runtime_requirements = &mut args.runtime_requirements;
    let chunk_ukey = args.chunk;

    if compilation
      .chunk_graph
      .get_number_of_entry_modules(chunk_ukey)
      > 0
    {
      let chunk = compilation
        .chunk_by_ukey
        .get(chunk_ukey)
        .expect("chunk do not exists");
      if let Some(startup_child_chunks) =
        chunk.get_children_of_type_in_order(&ChunkGroupOrderKey::Prefetch, compilation)
      {
        runtime_requirements.insert(RuntimeGlobals::PREFETCH_CHUNK);
        runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
        compilation.add_runtime_module(
          chunk_ukey,
          Box::new(ChunkPrefetchStartupRuntimeModule::new(startup_child_chunks)),
        )
      }
    }

    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let runtime_requirements = &mut args.runtime_requirements;
    let chunk_ukey = args.chunk;
    let chunk = compilation
      .chunk_by_ukey
      .get(chunk_ukey)
      .expect("chunk do not exists");
    let chunk_map = chunk.get_child_ids_by_orders_map(false, &compilation);

    if let Some(prefetch_map) = chunk_map.get(&ChunkGroupOrderKey::Prefetch) {
      runtime_requirements.insert(RuntimeGlobals::PREFETCH_CHUNK);
      compilation.add_runtime_module(
        chunk_ukey,
        Box::new(ChunkPrefetchTriggerRuntimeModule::new(
          prefetch_map.to_owned(),
        )),
      )
    }

    if let Some(preload_map) = chunk_map.get(&ChunkGroupOrderKey::Prefetch) {
      runtime_requirements.insert(RuntimeGlobals::PRELOAD_CHUNK);
      compilation.add_runtime_module(
        chunk_ukey,
        Box::new(ChunkPreloadTriggerRuntimeModule::new(
          preload_map.to_owned(),
        )),
      )
    }

    Ok(())
  }
}

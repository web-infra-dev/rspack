use std::ptr::NonNull;

use rspack_core::{ChunkUkey, Compilation, CompilationId};
use rspack_hook::define_hook;

#[derive(Debug, Clone)]
pub struct CreateScriptData {
  pub code: String,
  pub chunk: RuntimeModuleChunkWrapper,
}

#[derive(Debug, Clone)]
pub struct LinkPreloadData {
  pub code: String,
  pub chunk: RuntimeModuleChunkWrapper,
}

#[derive(Debug, Clone)]
pub struct LinkPrefetchData {
  pub code: String,
  pub chunk: RuntimeModuleChunkWrapper,
}

#[derive(Debug, Clone)]
pub struct RuntimeModuleChunkWrapper {
  pub chunk_ukey: ChunkUkey,
  pub compilation_id: CompilationId,
  pub compilation: NonNull<Compilation>,
}

unsafe impl Send for RuntimeModuleChunkWrapper {}

define_hook!(RuntimePluginCreateScript: AsyncSeriesWaterfall(data: CreateScriptData) -> CreateScriptData);
define_hook!(RuntimePluginLinkPreload: AsyncSeriesWaterfall(data: LinkPreloadData) -> LinkPreloadData);
define_hook!(RuntimePluginLinkPrefetch: AsyncSeriesWaterfall(data: LinkPrefetchData) -> LinkPrefetchData);

#[derive(Debug, Default)]
pub struct RuntimePluginHooks {
  pub create_script: RuntimePluginCreateScriptHook,
  pub link_preload: RuntimePluginLinkPreloadHook,
  pub link_prefetch: RuntimePluginLinkPrefetchHook,
}

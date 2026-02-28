use std::ptr::NonNull;

use rspack_core::{ChunkUkey, Compilation, CompilationId};
use rspack_hook::define_hook;
#[cfg(allocative)]
use rspack_util::allocative;

#[derive(Debug, Clone)]
pub struct CreateScriptData {
  pub code: String,
  pub chunk: RuntimeModuleChunkWrapper,
}

#[derive(Debug, Clone)]
pub struct CreateLinkData {
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

define_hook!(RuntimePluginCreateScript: SeriesWaterfall(data: CreateScriptData) -> CreateScriptData);
define_hook!(RuntimePluginCreateLink: SeriesWaterfall(data: CreateLinkData) -> CreateLinkData);
define_hook!(RuntimePluginLinkPreload: SeriesWaterfall(data: LinkPreloadData) -> LinkPreloadData);
define_hook!(RuntimePluginLinkPrefetch: SeriesWaterfall(data: LinkPrefetchData) -> LinkPrefetchData);

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct RuntimePluginHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub create_script: RuntimePluginCreateScriptHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub create_link: RuntimePluginCreateLinkHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub link_preload: RuntimePluginLinkPreloadHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub link_prefetch: RuntimePluginLinkPrefetchHook,
}

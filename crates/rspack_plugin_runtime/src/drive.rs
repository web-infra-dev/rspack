use std::ptr::NonNull;

use rspack_core::{Chunk, ChunkUkey, Compilation, CompilationId};
use rspack_hook::define_hook;
#[cfg(allocative)]
use rspack_util::allocative;

#[derive(Debug, Clone)]
pub struct CreateScriptData {
  pub code: String,
  pub chunk: RuntimeModuleChunkWrapper,
}

#[derive(Debug, Clone)]
pub struct CreateLinkData<'a> {
  pub code: String,
  pub chunk: &'a Chunk,
}

#[derive(Debug, Clone)]
pub struct LinkPreloadData<'a> {
  pub code: String,
  pub chunk: &'a Chunk,
}

#[derive(Debug, Clone)]
pub struct LinkPrefetchData<'a> {
  pub code: String,
  pub chunk: &'a Chunk,
}

#[derive(Debug, Clone)]
pub struct RuntimeModuleChunkWrapper {
  pub chunk_ukey: ChunkUkey,
  pub compilation_id: CompilationId,
  pub compilation: NonNull<Compilation>,
}

unsafe impl Send for RuntimeModuleChunkWrapper {}

define_hook!(RuntimePluginCreateScript: SeriesWaterfall(data: CreateScriptData) -> CreateScriptData);
define_hook!(RuntimePluginCreateLink: SeriesWaterfall(compilation: &Compilation, data: CreateLinkData<'_>) -> CreateLinkData<'_>);
define_hook!(RuntimePluginLinkPreload: SeriesWaterfall(compilation: &Compilation, data: LinkPreloadData<'_>) -> LinkPreloadData<'_>);
define_hook!(RuntimePluginLinkPrefetch: SeriesWaterfall(compilation: &Compilation, data: LinkPrefetchData<'_>) -> LinkPrefetchData<'_>);

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

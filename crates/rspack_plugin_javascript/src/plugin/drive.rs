use rspack_core::{
  AssetInfo, BoxModule, Chunk, ChunkInitFragments, ChunkUkey, Compilation, Module,
  ModuleIdentifier, rspack_sources::BoxSource,
};
use rspack_hash::RspackHash;
use rspack_hook::define_hook;
#[cfg(allocative)]
use rspack_util::allocative;

define_hook!(JavascriptModulesRenderChunk: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, source: &mut RenderSource));
define_hook!(JavascriptModulesRenderChunkContent: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey, asset_info: &mut AssetInfo) -> RenderSource);
define_hook!(JavascriptModulesRender: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, source: &mut RenderSource));
define_hook!(JavascriptModulesRenderStartup: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, module: &ModuleIdentifier, source: &mut RenderSource));
define_hook!(JavascriptModulesRenderModuleContent: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey,module: &dyn Module, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
define_hook!(JavascriptModulesRenderModuleContainer: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey,module: &dyn Module, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
define_hook!(JavascriptModulesRenderModulePackage: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, module: &dyn Module, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
define_hook!(JavascriptModulesChunkHash: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, hasher: &mut RspackHash));
define_hook!(JavascriptModulesInlineInRuntimeBailout: SeriesBail(compilation: &Compilation) -> String);
define_hook!(JavascriptModulesEmbedInRuntimeBailout: SeriesBail(compilation: &Compilation, module: &BoxModule, chunk: &Chunk) -> String);
define_hook!(JavascriptModulesStrictRuntimeBailout: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> String);

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct JavascriptModulesPluginHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub render_chunk: JavascriptModulesRenderChunkHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_chunk_content: JavascriptModulesRenderChunkContentHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render: JavascriptModulesRenderHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_startup: JavascriptModulesRenderStartupHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_module_content: JavascriptModulesRenderModuleContentHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_module_container: JavascriptModulesRenderModuleContainerHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_module_package: JavascriptModulesRenderModulePackageHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub chunk_hash: JavascriptModulesChunkHashHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub inline_in_runtime_bailout: JavascriptModulesInlineInRuntimeBailoutHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub embed_in_runtime_bailout: JavascriptModulesEmbedInRuntimeBailoutHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub strict_runtime_bailout: JavascriptModulesStrictRuntimeBailoutHook,
}

#[derive(Debug)]
pub struct RenderSource {
  pub source: BoxSource,
}

use rspack_core::{
  rspack_sources::BoxSource, BoxModule, Chunk, ChunkInitFragments, ChunkUkey, Compilation,
  ModuleIdentifier,
};
use rspack_hash::RspackHash;
use rspack_hook::define_hook;

define_hook!(JavascriptModulesRenderChunk: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, source: &mut RenderSource));
define_hook!(JavascriptModulesRenderChunkContent: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> RenderSource);
define_hook!(JavascriptModulesRender: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, source: &mut RenderSource));
define_hook!(JavascriptModulesRenderStartup: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, module: &ModuleIdentifier, source: &mut RenderSource));
define_hook!(JavascriptModulesRenderModuleContent: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey,module: &BoxModule, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
define_hook!(JavascriptModulesRenderModuleContainer: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey,module: &BoxModule, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
define_hook!(JavascriptModulesRenderModulePackage: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, module: &BoxModule, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
define_hook!(JavascriptModulesChunkHash: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, hasher: &mut RspackHash));
define_hook!(JavascriptModulesInlineInRuntimeBailout: SeriesBail(compilation: &Compilation) -> String);
define_hook!(JavascriptModulesEmbedInRuntimeBailout: SeriesBail(compilation: &Compilation, module: &BoxModule, chunk: &Chunk) -> String);
define_hook!(JavascriptModulesStrictRuntimeBailout: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> String);

#[derive(Debug, Default)]
pub struct JavascriptModulesPluginHooks {
  pub render_chunk: JavascriptModulesRenderChunkHook,
  pub render_chunk_content: JavascriptModulesRenderChunkContentHook,
  pub render: JavascriptModulesRenderHook,
  pub render_startup: JavascriptModulesRenderStartupHook,
  pub render_module_content: JavascriptModulesRenderModuleContentHook,
  pub render_module_container: JavascriptModulesRenderModuleContainerHook,
  pub render_module_package: JavascriptModulesRenderModulePackageHook,
  pub chunk_hash: JavascriptModulesChunkHashHook,
  pub inline_in_runtime_bailout: JavascriptModulesInlineInRuntimeBailoutHook,
  pub embed_in_runtime_bailout: JavascriptModulesEmbedInRuntimeBailoutHook,
  pub strict_runtime_bailout: JavascriptModulesStrictRuntimeBailoutHook,
}

#[derive(Debug)]
pub struct RenderSource {
  pub source: BoxSource,
}

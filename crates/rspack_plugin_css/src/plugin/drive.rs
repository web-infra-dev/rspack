use rspack_core::{rspack_sources::BoxSource, ChunkUkey, Compilation, Module};
use rspack_hook::define_hook;

define_hook!(CssModulesRenderModulePackage: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, module: &dyn Module, source: &mut CssModulesRenderSource),tracing=false);

#[derive(Debug, Default)]
pub struct CssModulesPluginHooks {
  pub render_module_package: CssModulesRenderModulePackageHook,
}

#[derive(Debug)]
pub struct CssModulesRenderSource {
  pub source: BoxSource,
}

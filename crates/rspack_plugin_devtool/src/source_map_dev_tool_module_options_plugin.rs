use rspack_collections::IdentifierMap;
use rspack_core::{
  BoxModule, ChunkUkey, Compilation, CompilationBuildModule, CompilationId,
  CompilationRuntimeModule, CompilerId, ModuleIdentifier, Plugin, RuntimeModule,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::source_map::SourceMapKind;

pub struct SourceMapDevToolModuleOptionsPluginOptions {
  pub source_map_kind: SourceMapKind,
}

#[plugin]
#[derive(Debug)]
pub struct SourceMapDevToolModuleOptionsPlugin {
  source_map_kind: SourceMapKind,
}

impl SourceMapDevToolModuleOptionsPlugin {
  pub fn new(options: SourceMapDevToolModuleOptionsPluginOptions) -> Self {
    Self::new_inner(options.source_map_kind)
  }
}

#[plugin_hook(CompilationBuildModule for SourceMapDevToolModuleOptionsPlugin)]
async fn build_module(
  &self,
  _compiler_id: CompilerId,
  _compilation_id: CompilationId,
  module: &mut BoxModule,
) -> Result<()> {
  module.set_source_map_kind(self.source_map_kind);
  Ok(())
}

#[plugin_hook(CompilationRuntimeModule for SourceMapDevToolModuleOptionsPlugin)]
async fn runtime_module(
  &self,
  _compilation: &Compilation,
  module_identifier: &ModuleIdentifier,
  _chunk: &ChunkUkey,
  runtime_modules: &mut IdentifierMap<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let Some(runtime_module) = runtime_modules.get_mut(module_identifier) else {
    return Ok(());
  };
  runtime_module.set_source_map_kind(self.source_map_kind);
  Ok(())
}

impl Plugin for SourceMapDevToolModuleOptionsPlugin {
  fn name(&self) -> &'static str {
    "SourceMapDevToolModuleOptionsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .build_module
      .tap(build_module::new(self));
    ctx
      .compilation_hooks
      .runtime_module
      .tap(runtime_module::new(self));
    Ok(())
  }
}

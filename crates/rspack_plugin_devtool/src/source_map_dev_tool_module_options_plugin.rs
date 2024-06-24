use rspack_core::{
  ApplyContext, BoxModule, ChunkUkey, Compilation, CompilationBuildModule,
  CompilationRuntimeModule, CompilerOptions, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::source_map::SourceMapKind;

pub struct SourceMapDevToolModuleOptionsPluginOptions {
  pub module: bool,
  pub cheap: bool,
}

#[plugin]
#[derive(Debug)]
pub struct SourceMapDevToolModuleOptionsPlugin {
  module: bool,
  cheap: bool,
}

impl SourceMapDevToolModuleOptionsPlugin {
  pub fn new(options: SourceMapDevToolModuleOptionsPluginOptions) -> Self {
    Self::new_inner(options.module, options.cheap)
  }
}

#[plugin_hook(CompilationBuildModule for SourceMapDevToolModuleOptionsPlugin)]
async fn build_module(&self, module: &mut BoxModule) -> Result<()> {
  if self.module {
    module.set_source_map_kind(SourceMapKind::SourceMap);
  } else {
    module.set_source_map_kind(SourceMapKind::SimpleSourceMap);
  }
  if self.cheap {
    module.set_source_map_kind(*module.get_source_map_kind() | SourceMapKind::Cheap)
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeModule for SourceMapDevToolModuleOptionsPlugin)]
async fn runtime_module(
  &self,
  compilation: &mut Compilation,
  module: &ModuleIdentifier,
  _chunk: &ChunkUkey,
) -> Result<()> {
  let Some(module) = compilation.runtime_modules.get_mut(module) else {
    return Ok(());
  };
  if self.module {
    module.set_source_map_kind(SourceMapKind::SourceMap);
  } else {
    module.set_source_map_kind(SourceMapKind::SimpleSourceMap);
  }
  if self.cheap {
    module.set_source_map_kind(*module.get_source_map_kind() | SourceMapKind::Cheap)
  }
  Ok(())
}

impl Plugin for SourceMapDevToolModuleOptionsPlugin {
  fn name(&self) -> &'static str {
    "SourceMapDevToolModuleOptionsPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .build_module
      .tap(build_module::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_module
      .tap(runtime_module::new(self));
    Ok(())
  }
}

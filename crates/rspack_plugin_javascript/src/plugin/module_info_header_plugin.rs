use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawStringSource, SourceExt};
use rspack_core::{
  ApplyContext, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{JavascriptModulesRenderModuleContent, JsPlugin, RenderSource};

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleInfoHeaderPlugin {
  verbose: bool,
}

#[plugin_hook(CompilerCompilation for ModuleInfoHeaderPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  println!("here? ModuleInfoHeaderPlugin");

  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks.render_module_content.tap(render_module_content::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for ModuleInfoHeaderPlugin)]
fn render_module_content(
  &self,
  _compilation: &Compilation,

  _module: &BoxModule,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let mut new_source: ConcatSource = Default::default();

  new_source.add(RawStringSource::from("/* just a try*/"));

  println!("lalala");

  new_source.add(render_source.source.clone());

  render_source.source = new_source.boxed();

  Ok(())
}

#[async_trait]
impl Plugin for ModuleInfoHeaderPlugin {
  fn name(&self) -> &'static str {
    "rspack.ModuleInfoHeaderPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> Result<()> {
    println!("ModuleInfoHeaderPlugin applied?");

    _ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}

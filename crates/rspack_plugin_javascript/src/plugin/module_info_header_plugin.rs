use std::sync::LazyLock;

use async_trait::async_trait;
use regex::Regex;
use rspack_collections::Identifiable;
use rspack_core::rspack_sources::{ConcatSource, RawStringSource, SourceExt};
use rspack_core::{
  ApplyContext, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{JavascriptModulesRenderModulePackage, JsPlugin, RenderSource};

static COMMENT_END_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\*/").expect("should init regex"));

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleInfoHeaderPlugin {
  verbose: bool,
}

impl ModuleInfoHeaderPlugin {
  pub fn generate_header(module: &BoxModule, compilation: &Compilation) -> String {
    let req = module.readable_identifier(&compilation.options.context);
    let req = COMMENT_END_REGEX.replace_all(&req, "*_/");

    let req_stars_str = "*".repeat(req.len());

    format!("\n/*!****{req_stars_str}****!*\\\n  !*** {req} ***!\n  \\****{req_stars_str}****/\n")
  }
}

#[plugin_hook(CompilerCompilation for ModuleInfoHeaderPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  println!("here? ModuleInfoHeaderPlugin");

  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .render_module_package
    .tap(render_module_package::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModulePackage for ModuleInfoHeaderPlugin)]
fn render_module_package(
  &self,
  compilation: &Compilation,
  module: &BoxModule,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let mut new_source: ConcatSource = Default::default();

  dbg!(&module.identifier().to_string());
  dbg!(&module.module_type());

  new_source.add(RawStringSource::from(
    ModuleInfoHeaderPlugin::generate_header(module, compilation),
  ));

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

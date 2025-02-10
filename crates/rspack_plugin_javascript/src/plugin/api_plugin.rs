use rspack_core::{
  ApplyContext, BoxModule, ChunkInitFragments, Compilation, CompilationParams, CompilerCompilation,
  CompilerOptions, InitFragmentExt, InitFragmentKey, InitFragmentStage, NormalInitFragment, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{JavascriptModulesRenderModuleContent, JsPlugin, RenderSource};

#[plugin]
#[derive(Debug, Default)]
pub struct APIPlugin;

#[plugin_hook(CompilerCompilation for APIPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .render_module_content
    .tap(render_module_content::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for APIPlugin)]
fn render_module_content(
  &self,
  compilation: &Compilation,
  module: &BoxModule,
  _source: &mut RenderSource,
  init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  if module.build_info().need_create_require {
    let need_prefix = compilation
      .options
      .output
      .environment
      .supports_node_prefix_for_core_modules();

    init_fragments.push(
      NormalInitFragment::new(
        format!(
          "import {{ createRequire as __WEBPACK_EXTERNAL_createRequire }} from \"{}\";\n",
          if need_prefix { "node:module" } else { "module" }
        ),
        InitFragmentStage::StageESMImports,
        0,
        InitFragmentKey::ModuleExternal("node-commonjs".to_string()),
        None,
      )
      .boxed(),
    );
  }
  Ok(())
}

impl Plugin for APIPlugin {
  fn name(&self) -> &'static str {
    "rspack.APIPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}

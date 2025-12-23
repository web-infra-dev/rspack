use rspack_core::{
  ChunkInitFragments, ChunkUkey, Compilation, CompilationParams, CompilerCompilation,
  InitFragmentExt, InitFragmentKey, InitFragmentStage, Module, NormalInitFragment, Plugin,
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
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .write()
    .await
    .render_module_content
    .tap(render_module_content::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for APIPlugin,tracing=false)]
async fn render_module_content(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  module: &dyn Module,
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
          "import {{ createRequire as __rspack_createRequire }} from \"{}\";\n{} __rspack_createRequire_require = __rspack_createRequire({}.url);\n",
          if need_prefix { "node:module" } else { "module" },
          if compilation.options.output.environment.supports_const() {
            "const"
          } else {
            "var"
          },
          compilation.options.output.import_meta_name
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

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    Ok(())
  }
}

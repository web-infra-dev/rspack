use std::sync::Arc;

use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerCompilation, CompilerOptions,
  InitFragmentExt, InitFragmentKey, InitFragmentStage, NormalInitFragment, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{
  JavascriptModulesPluginPlugin, JsPlugin, PluginRenderJsModuleContentOutput,
  RenderJsModuleContentArgs,
};

#[derive(Debug, Default)]
struct APIJavascriptModulesPluginPlugin;

impl JavascriptModulesPluginPlugin for APIJavascriptModulesPluginPlugin {
  fn render_module_content<'a>(
    &'a self,
    mut args: RenderJsModuleContentArgs<'a>,
  ) -> PluginRenderJsModuleContentOutput<'a> {
    if let Some(build_info) = &args.module.build_info()
      && build_info.need_create_require
    {
      args.chunk_init_fragments.push(
        NormalInitFragment::new(
          "import { createRequire as __WEBPACK_EXTERNAL_createRequire } from 'module';\n"
            .to_string(),
          InitFragmentStage::StageHarmonyImports,
          0,
          InitFragmentKey::ExternalModule("node-commonjs".to_string()),
          None,
        )
        .boxed(),
      );
    }
    Ok(args)
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct APIPlugin {
  js_plugin: Arc<APIJavascriptModulesPluginPlugin>,
}

#[plugin_hook(CompilerCompilation for APIPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut drive = JsPlugin::get_compilation_drives_mut(compilation);
  drive.add_plugin(self.js_plugin.clone());
  Ok(())
}

impl Plugin for APIPlugin {
  fn name(&self) -> &'static str {
    "rspack.APIPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}

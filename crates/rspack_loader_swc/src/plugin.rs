use std::sync::Arc;

use rspack_core::{
  Compilation, CompilationFinishModules, CompilationParams, CompilerCompilation, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[derive(Debug)]
pub struct SwcDtsEmitOptions {
  pub root_dir: String,
}

#[plugin]
#[derive(Debug)]
pub struct PluginSwcDtsEmit {
  pub(crate) options: Arc<SwcDtsEmitOptions>,
}

impl Eq for PluginSwcDtsEmit {}

impl PartialEq for PluginSwcDtsEmit {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.options, &other.options)
  }
}

const PLUGIN_NAME: &str = "rspack.SwcDtsEmitPlugin";

impl PluginSwcDtsEmit {
  pub fn new(options: SwcDtsEmitOptions) -> Self {
    Self::new_inner(Arc::new(options))
  }
}

#[plugin_hook(CompilationFinishModules for PluginSwcDtsEmit)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  for (_, a) in module_graph.modules() {
    let meta = &a.build_info().expect("fuck").parse_meta;
    dbg!(meta);
  }
  Ok(())
}

// #[plugin_hook(CompilerCompilation for PluginSwcDtsEmit)]
// async fn compilation(
//   &self,
//   compilation: &mut Compilation,
//   params: &mut CompilationParams,
// ) -> Result<()> {
//   Ok(())
// }

impl Plugin for PluginSwcDtsEmit {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    // ctx
    //   .context
    //   .compiler_hooks
    //   .compilation
    //   .tap(compilation::new(self));

    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));

    Ok(())
  }
}

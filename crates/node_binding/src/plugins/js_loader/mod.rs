mod context;
mod resolver;
mod scheduler;
use std::fmt::Debug;

pub use context::{JsLoaderContext, JsLoaderItem};
use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use once_cell::sync::OnceCell;
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerId, CompilerOptions,
  CompilerThisCompilation, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use tokio::sync::RwLock;

pub type JsLoaderRunner =
  ThreadsafeFunction<JsLoaderContext, Promise<JsLoaderContext>, JsLoaderContext, false, true, 0>;

pub type JsLoaderRunnerGetter = ThreadsafeFunction<
  External<CompilerId>,
  &'static mut External<Option<JsLoaderRunner>>,
  External<CompilerId>,
  false,
  true,
  1,
>;

#[plugin]
pub(crate) struct JsLoaderRspackPlugin {
  compiler_id: OnceCell<CompilerId>,
  pub(crate) runner_getter: JsLoaderRunnerGetter,
  pub(crate) runner: RwLock<Option<JsLoaderRunner>>,
}

impl JsLoaderRspackPlugin {
  pub fn new(runner_getter: JsLoaderRunnerGetter) -> Self {
    Self::new_inner(Default::default(), runner_getter, RwLock::new(None))
  }
}

impl Debug for JsLoaderRspackPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("LoaderResolver").finish()
  }
}

#[plugin_hook(CompilerThisCompilation for JsLoaderRspackPlugin)]
async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let compiler_id = compilation.compiler_id();
  let _ = self.compiler_id.get_or_init(|| compiler_id);
  Ok(())
}

impl Plugin for JsLoaderRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsLoaderRspackPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));

    ctx
      .context
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolver::resolve_loader::new(self));

    ctx
      .context
      .normal_module_hooks
      .loader_should_yield
      .tap(scheduler::loader_should_yield::new(self));

    ctx
      .context
      .normal_module_hooks
      .loader_yield
      .tap(scheduler::loader_yield::new(self));
    Ok(())
  }
}

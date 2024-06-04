mod context;
mod resolver;
mod scheduler;

use std::fmt::Debug;

pub use context::JsLoaderContext;
use napi::bindgen_prelude::*;
use rspack_core::{ApplyContext, CompilerOptions, Plugin, PluginContext};
use rspack_error::Result;
use rspack_hook::plugin;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

pub type JsLoaderRunner = ThreadsafeFunction<JsLoaderContext, Promise<JsLoaderContext>>;

#[plugin]
pub(crate) struct JsLoaderRspackPlugin {
  pub(crate) runner: JsLoaderRunner,
}

impl JsLoaderRspackPlugin {
  pub fn new(runner: JsLoaderRunner) -> Self {
    Self::new_inner(runner)
  }
}

impl Debug for JsLoaderRspackPlugin {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("JsLoaderResolver").finish()
  }
}

impl Plugin for JsLoaderRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsLoaderRspackPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
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

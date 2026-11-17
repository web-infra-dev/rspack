mod bridge;
mod context;
mod resolver;
mod result;
pub(crate) mod worker;

use std::{
  fmt::Debug,
  sync::{Arc, LazyLock, Mutex, Weak},
};

pub use context::{JsLoaderContext, JsLoaderDependencies, JsLoaderItem};
use rspack_core::{Compilation, CompilationParams, CompilerId, CompilerThisCompilation, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::JsLoaderRuntime;
use rustc_hash::FxHashMap;

static RUNTIMES: LazyLock<Mutex<FxHashMap<CompilerId, Weak<JsLoaderRuntime>>>> =
  LazyLock::new(Default::default);

pub(crate) fn release_runtime(compiler_id: CompilerId) {
  RUNTIMES
    .lock()
    .expect("loader runtimes lock poisoned")
    .remove(&compiler_id);
}

fn runtime(compiler_id: CompilerId) -> Option<Arc<JsLoaderRuntime>> {
  RUNTIMES
    .lock()
    .expect("loader runtimes lock poisoned")
    .get(&compiler_id)
    .and_then(Weak::upgrade)
}

#[plugin]
pub(crate) struct JsLoaderRspackPlugin {
  pub(crate) runtime: Arc<JsLoaderRuntime>,
  pub(crate) resolved_loaders: resolver::LoaderResolveCache,
}

impl JsLoaderRspackPlugin {
  pub fn new(main_object_handle: u32) -> Self {
    Self::new_inner(
      bridge::loader_runtime(main_object_handle),
      Default::default(),
    )
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
  RUNTIMES
    .lock()
    .expect("loader runtimes lock poisoned")
    .insert(compilation.compiler_id(), Arc::downgrade(&self.runtime));
  self
    .resolved_loaders
    .lock()
    .expect("loader resolution cache lock poisoned")
    .clear();
  Ok(())
}

impl Plugin for JsLoaderRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsLoaderRspackPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));

    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolver::resolve_loader::new(self));

    Ok(())
  }
}

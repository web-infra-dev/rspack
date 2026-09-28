#[cfg(allocative)]
use rspack_util::allocative;

mod cache;
mod context;
mod resolver;
mod scheduler;

use std::{
  fmt::Debug,
  sync::{Arc, Mutex},
};

pub use cache::{JsLoaderCache, JsLoaderCacheEntry};
pub use context::{JsLoaderContext, JsLoaderDependencies, JsLoaderItem};
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerClose, CompilerEmit, CompilerId,
  CompilerOptions, CompilerThisCompilation, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;
use tokio::sync::{OnceCell, RwLock};

use crate::{COMPILER_REFERENCES, error::RspackResultToNapiResultExt};

pub type JsLoaderRunner = ThreadsafeFunction<
  JsLoaderContext,
  Promise<JsLoaderContext>,
  JsLoaderContext,
  Status,
  false,
  true,
  0,
>;

type JsLoaderRunnerGetterTsfn = ThreadsafeFunction<
  External<CompilerId>,
  Unknown<'static>,
  External<CompilerId>,
  Status,
  false,
  true,
>;

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct JsLoaderRunnerGetter {
  ts_fn: Mutex<Option<JsLoaderRunnerGetterTsfn>>,
}

impl JsLoaderRunnerGetter {
  pub fn new(env: &Env) -> napi::Result<Self> {
    // The callback only looks up a weak compiler reference. The owned TSFN is
    // released on close (or plugin drop), without retaining the compiler through JS.
    let getter: Function<External<CompilerId>, Unknown<'static>> = env
      .create_function_from_closure("get_loader_runner", |ctx| {
        let compiler_id = ctx.get::<&External<CompilerId>>(0)?;
        COMPILER_REFERENCES.with(|ref_cell| {
          let references = ref_cell.borrow();
          let weak_reference = references.get(&**compiler_id).ok_or_else(|| {
            napi::Error::from_reason(
              "Failed to get loader runner: the Compiler has been garbage collected by JavaScript.",
            )
          })?;
          let compiler_object = unsafe {
            let value = ToNapiValue::to_napi_value(ctx.env.raw(), weak_reference.clone())?;
            Object::from_napi_value(ctx.env.raw(), value)?
          };
          compiler_object.get_named_property("_runLoader")
        })
      })?;
    let ts_fn = getter
      .build_threadsafe_function::<External<CompilerId>>()
      .weak::<true>()
      .callee_handled::<false>()
      .build()?;
    Ok(Self {
      ts_fn: Mutex::new(Some(ts_fn)),
    })
  }

  pub async fn call(&self, compiler_id: &CompilerId) -> napi::Result<JsLoaderRunner> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let status = {
      let ts_fn = self.ts_fn.lock().expect("should get lock");
      let ts_fn = ts_fn
        .as_ref()
        .ok_or_else(|| napi::Error::from_reason("Loader runner getter has already been closed"))?;
      ts_fn.call_with_return_value(
        External::new(*compiler_id),
        ThreadsafeFunctionCallMode::NonBlocking,
        move |result, env| {
          // Convert on the owning JS thread, before sending the runner to Rust.
          let result = result
            .and_then(|value| unsafe { JsLoaderRunner::from_napi_value(env.raw(), value.raw()) });
          let _ = tx.send(result);
          Ok(())
        },
      )
    };
    if status != Status::Ok {
      return Err(napi::Error::from_status(status));
    }
    rx.await.to_napi_result()?
  }

  pub fn close(&self) {
    self.ts_fn.lock().expect("should get lock").take();
  }
}

#[plugin]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub(crate) struct JsLoaderRspackPlugin {
  compiler_id: once_cell::sync::OnceCell<CompilerId>,
  pub(crate) runner_getter: JsLoaderRunnerGetter,
  /// This complex data structure is used to avoid deadlock when running loaders which contain `importModule`
  /// See: https://github.com/web-infra-dev/rspack/pull/10632
  pub(crate) runner: Mutex<Arc<tokio::sync::OnceCell<JsLoaderRunner>>>,
  pub(crate) loaders_without_pitch: RwLock<FxHashSet<String>>,
}

impl JsLoaderRspackPlugin {
  pub fn new(runner_getter: JsLoaderRunnerGetter) -> Self {
    Self::new_inner(
      Default::default(),
      runner_getter,
      Mutex::default(),
      RwLock::new(FxHashSet::default()),
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
  let compiler_id = compilation.compiler_id();
  let _ = self.compiler_id.get_or_init(|| compiler_id);
  Ok(())
}

#[plugin_hook(CompilerEmit for JsLoaderRspackPlugin)]
async fn done(&self, _compilation: &mut Compilation) -> Result<()> {
  *self.runner.lock().expect("should get lock") = Arc::new(OnceCell::new());
  Ok(())
}

// Release loader callbacks before a later close hook can fail. The binding
// waits for in-flight builds, so no loader can still be using these handles.
#[plugin_hook(CompilerClose for JsLoaderRspackPlugin, stage = i32::MIN)]
async fn close(&self, _compilation: &Compilation) -> Result<()> {
  // Emit normally clears the runner, but builds that skip emit still own it.
  *self.runner.lock().expect("should get lock") = Arc::new(OnceCell::new());
  self.runner_getter.close();
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

    ctx
      .normal_module_hooks
      .loader_yield
      .tap(scheduler::loader_yield::new(self));

    // TODO: tap compiler done hook will be better.
    ctx.compiler_hooks.emit.tap(done::new(self));
    ctx.compiler_hooks.close.tap(close::new(self));
    Ok(())
  }
}

mod context;
mod resolver;
mod scheduler;

use std::{
  ffi::c_void,
  fmt::Debug,
  ptr,
  sync::{Arc, Mutex},
};

pub use context::{JsLoaderContext, JsLoaderItem};
use napi::{
  bindgen_prelude::*,
  sys::{napi_call_threadsafe_function, napi_threadsafe_function},
  threadsafe_function::ThreadsafeFunction,
};
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerEmit, CompilerId, CompilerOptions,
  CompilerThisCompilation, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashSet;
use tokio::sync::{OnceCell, RwLock};

use crate::{RspackResultToNapiResultExt, COMPILER_REFERENCES};

pub type JsLoaderRunner = ThreadsafeFunction<
  JsLoaderContext,
  Promise<JsLoaderContext>,
  JsLoaderContext,
  Status,
  false,
  true,
  0,
>;

struct JsLoaderRunnerGetterData {
  compiler_id: CompilerId,
  tx: tokio::sync::oneshot::Sender<napi::Result<JsLoaderRunner>>,
}

extern "C" fn napi_js_callback(
  env: sys::napi_env,
  _js_callback: sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  // env can be null when shutting down
  if env.is_null() {
    return;
  }

  let data: Box<JsLoaderRunnerGetterData> = unsafe { Box::from_raw(data.cast()) };
  let JsLoaderRunnerGetterData { compiler_id, tx } = *data;

  let result = COMPILER_REFERENCES.with(|ref_cell| {
    if let Some(weak_reference) = ref_cell.borrow().get(&compiler_id) {
      let compiler_object = unsafe {
        let napi_value = ToNapiValue::to_napi_value(env, weak_reference.clone())?;
        Object::from_napi_value(env, napi_value)?
      };
      let run_loader = compiler_object
        .get_named_property::<Function<JsLoaderContext, Promise<JsLoaderContext>>>("_runLoader")?;
      let ts_fn: JsLoaderRunner = run_loader
        .build_threadsafe_function::<JsLoaderContext>()
        .weak::<true>()
        .callee_handled::<false>()
        .max_queue_size::<0>()
        .build()?;
      Ok(ts_fn)
    } else {
      Err(napi::Error::from_reason(
        "Failed to get loader runner: the Compiler has been garbage collected by JavaScript.",
      ))
    }
  });
  let _ = tx.send(result);
}

pub struct JsLoaderRunnerGetter {
  ts_fn: napi_threadsafe_function,
}

unsafe impl Send for JsLoaderRunnerGetter {}
unsafe impl Sync for JsLoaderRunnerGetter {}

impl JsLoaderRunnerGetter {
  pub fn new(env: &Env) -> napi::Result<Self> {
    let raw_env = env.raw();

    let mut async_resource_name = ptr::null_mut();
    check_status!(
      unsafe {
        sys::napi_create_string_utf8(
          raw_env,
          c"delete_reference_ts_fn".as_ptr(),
          16,
          &mut async_resource_name,
        )
      },
      "Failed to create async resource name"
    )?;

    let mut ts_fn = ptr::null_mut();
    check_status!(
      unsafe {
        sys::napi_create_threadsafe_function(
          raw_env,
          ptr::null_mut(),
          ptr::null_mut(),
          async_resource_name,
          0,
          1,
          ptr::null_mut(),
          None,
          ptr::null_mut(),
          Some(napi_js_callback),
          &mut ts_fn,
        )
      },
      "Failed to create threadsafe function"
    )?;
    check_status!(unsafe { sys::napi_unref_threadsafe_function(raw_env, ts_fn) })?;
    Ok(Self { ts_fn })
  }

  pub async fn call(&self, compiler_id: &CompilerId) -> napi::Result<JsLoaderRunner> {
    let (tx, rx) = tokio::sync::oneshot::channel::<napi::Result<JsLoaderRunner>>();

    let data = JsLoaderRunnerGetterData {
      compiler_id: *compiler_id,
      tx,
    };
    unsafe {
      let _ = napi_call_threadsafe_function(
        self.ts_fn,
        Box::into_raw(Box::new(data)).cast(),
        sys::ThreadsafeFunctionCallMode::nonblocking,
      );
    }
    let result = rx.await.to_napi_result()?;
    let loader_runner = result?;
    Ok(loader_runner)
  }
}

#[plugin]
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

    // TODO: tap compiler done hook will be better.
    ctx.context.compiler_hooks.emit.tap(done::new(self));
    Ok(())
  }
}

mod context;
mod resolver;
mod scheduler;

use std::{ffi::c_void, fmt::Debug, ptr};

pub use context::{JsLoaderContext, JsLoaderItem};
use napi::{
  bindgen_prelude::*,
  sys::{napi_call_threadsafe_function, napi_threadsafe_function, napi_value},
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction},
};
use once_cell::sync::OnceCell;
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerEmit, CompilerId, CompilerOptions,
  CompilerThisCompilation, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use tokio::sync::RwLock;

use crate::{RspackResultToNapiResultExt, COMPILER_REFERENCES};

pub type JsLoaderRunner = ThreadsafeFunction<
  (
    JsLoaderContext,
    tokio::sync::oneshot::Sender<JsLoaderContext>,
  ),
  (),
  FnArgs<(JsLoaderContext, napi_value)>,
  false,
  true,
  0,
>;

unsafe extern "C" fn raw_done(
  env: sys::napi_env,
  cb_info: sys::napi_callback_info,
) -> sys::napi_value {
  handle_done_callback(env, cb_info).unwrap_or_else(|err| throw_error(env, err, "Error in done"))
}

#[inline(always)]
fn handle_done_callback(
  env: sys::napi_env,
  cb_info: sys::napi_callback_info,
) -> napi::Result<sys::napi_value> {
  let mut callback_values = [ptr::null_mut()];
  let mut data = ptr::null_mut();
  check_status!(
    unsafe {
      sys::napi_get_cb_info(
        env,
        cb_info,
        &mut 1,
        callback_values.as_mut_ptr(),
        ptr::null_mut(),
        &mut data,
      )
    },
    "Get callback info from loader runner callback failed"
  )?;

  let tx: Box<tokio::sync::oneshot::Sender<JsLoaderContext>> =
    unsafe { Box::from_raw(data.cast()) };

  let value = unsafe { FromNapiValue::from_napi_value(env, callback_values[0]) }?;
  let _ = tx.send(value);

  Ok(ptr::null_mut())
}

#[inline(never)]
fn throw_error(env: sys::napi_env, err: Error, default_msg: &str) -> sys::napi_value {
  const GENERIC_FAILURE: &str = "GenericFailure\0";
  let code = if err.status.as_ref().is_empty() {
    GENERIC_FAILURE
  } else {
    err.status.as_ref()
  };
  let mut code_string = ptr::null_mut();
  let msg = if err.reason.is_empty() {
    default_msg
  } else {
    err.reason.as_ref()
  };
  let mut msg_string = ptr::null_mut();
  let mut err = ptr::null_mut();
  unsafe {
    sys::napi_create_string_latin1(
      env,
      code.as_ptr().cast(),
      code.len() as isize,
      &mut code_string,
    );
    sys::napi_create_string_utf8(
      env,
      msg.as_ptr().cast(),
      msg.len() as isize,
      &mut msg_string,
    );
    sys::napi_create_error(env, code_string, msg_string, &mut err);
    sys::napi_throw(env, err);
  };
  ptr::null_mut()
}

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
        .get_named_property::<Function<FnArgs<(JsLoaderContext, napi_value)>, ()>>("_runLoader")?;
      let ts_fn: JsLoaderRunner = run_loader
        .build_threadsafe_function()
        .weak::<true>()
        .callee_handled::<false>()
        .max_queue_size::<0>()
        .build_callback(
          |ctx: ThreadsafeCallContext<(
            JsLoaderContext,
            tokio::sync::oneshot::Sender<JsLoaderContext>,
          )>| {
            let context = ctx.value.0;
            let sender = ctx.value.1;
            let mut done = ptr::null_mut();
            const DONE: &[u8; 5] = b"done\0";
            let data = Box::into_raw(Box::new(sender)).cast();
            check_status!(
              unsafe {
                sys::napi_create_function(
                  ctx.env.raw(),
                  DONE.as_ptr().cast(),
                  4,
                  Some(raw_done),
                  data,
                  &mut done,
                )
              },
              "Create then function for PromiseRaw failed"
            )?;
            Ok(FnArgs::from((context, done)))
          },
        )?;
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

#[plugin_hook(CompilerEmit for JsLoaderRspackPlugin)]
async fn done(&self, _compilation: &mut Compilation) -> Result<()> {
  let mut write_guard = self.runner.write().await;
  *write_guard = None;
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

use std::sync::Arc;

use napi::{
  Env, Status,
  bindgen_prelude::{
    External, ExternalRef, FromNapiValue, Function, JsObjectValue, Object, Promise, Reference,
    WeakReference,
  },
  threadsafe_function::ThreadsafeFunction,
};
use rspack_core::CompilerId;
use rspack_error::ToStringResultToRspackResultExt;
use rspack_plugin_rsc::{ClientCompilerHandle, ReactClientPluginOptions};

use crate::JsCompiler;

#[napi(object, object_to_js = false)]
pub struct JsClientCompilerHandle {
  pub compile: ThreadsafeFunction<(), Promise<()>, (), Status, false, true, 0>,
}

impl From<JsClientCompilerHandle> for ClientCompilerHandle {
  fn from(value: JsClientCompilerHandle) -> Self {
    let ts_fn = Arc::new(value.compile);

    ClientCompilerHandle::new(Box::new(move || {
      let ts_fn = ts_fn.clone();
      Box::pin(async move {
        println!("Calling client compiler compile from JS");
        let promise = ts_fn.call_async(()).await.to_rspack_result()?;
        promise.await;
        println!("Calling client compiler compile from JS end");
        Ok(())
      })
    }))
  }
}

type GetServerCompilerId = Box<dyn Fn() -> CompilerId + Sync + Send>;

pub struct JsReactClientPluginOptions {
  pub get_server_compiler_id:
    ThreadsafeFunction<(), &'static External<CompilerId>, (), Status, false, true, 0>,
}

impl FromNapiValue for JsReactClientPluginOptions {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let obj = unsafe { Object::from_napi_value(env, napi_val)? };

    let js_fn = obj.get_named_property::<Function<'static, (), &'static External<CompilerId>>>(
      "getServerCompilerId",
    )?;

    let ts_fn = js_fn
      .build_threadsafe_function::<()>()
      .callee_handled::<false>()
      .max_queue_size()
      .weak::<true>()
      .build()?;

    Ok(Self {
      get_server_compiler_id: ts_fn,
    })
  }
}

impl From<JsReactClientPluginOptions> for ReactClientPluginOptions {
  fn from(value: JsReactClientPluginOptions) -> Self {
    let ts_fn = Arc::new(value.get_server_compiler_id);

    Self {
      get_server_compiler_id: Box::new(move || {
        let ts_fn = ts_fn.clone();
        Box::pin(async move {
          let external = ts_fn.call_async(()).await.to_rspack_result()?;
          Ok(**external)
        })
      }),
    }
  }
}

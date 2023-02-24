use std::ffi::CStr;
use std::io::Write;
use std::ptr;

use futures::Future;
use napi::bindgen_prelude::*;
use napi::{check_status, Env, Error, JsFunction, JsUnknown, NapiRaw};
use napi_derive::napi;
use once_cell::sync::OnceCell;
use rspack_error::CatchUnwindFuture;
use rspack_napi_shared::threadsafe_function::{
  ThreadSafeContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use rspack_napi_shared::{Result, RspackErrorExt, RspackResultExt};

static CUSTOM_TRACE_SUBSCRIBER: OnceCell<bool> = OnceCell::new();

#[napi]
pub fn init_custom_trace_subscriber(
  mut env: Env,
  // trace_out_file_path: Option<String>,
) -> Result<()> {
  CUSTOM_TRACE_SUBSCRIBER.get_or_init(|| {
    let layer = std::env::var("layer").unwrap_or("logger".to_string());
    let guard = match layer.as_str() {
      "chrome" => rspack_tracing::enable_tracing_by_env_with_chrome_layer(),
      "logger" => {
        rspack_tracing::enable_tracing_by_env();
        None
      }
      _ => panic!("not supported layer type:{layer}"),
    };
    if let Some(guard) = guard {
      env
        .add_env_cleanup_hook(guard, |flush_guard| {
          flush_guard.flush();
          drop(flush_guard);
        })
        .expect("Should able to initialize cleanup for custom trace subscriber");
    }
    true
  });

  Ok(())
}

pub fn callbackify<R, F>(env: Env, f: JsFunction, fut: F) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  let ptr = unsafe { f.raw() };

  let tsfn = ThreadsafeFunction::<Result<R>, ()>::create(env.raw(), ptr, 0, |ctx| {
    let ThreadSafeContext {
      value,
      env,
      callback,
      ..
    } = ctx;

    let argv = match value {
      Ok(value) => {
        let val = unsafe { R::to_napi_value(env.raw(), value)? };
        let js_value = unsafe { JsUnknown::from_napi_value(env.raw(), val)? };
        vec![env.get_null()?.into_unknown(), js_value]
      }
      Err(err) => {
        vec![JsError::from(err).into_unknown(env)]
      }
    };

    callback.call(None, &argv)?;

    Ok(())
  })
  .into_napi_result()?;

  napi::bindgen_prelude::spawn(async move {
    let fut = CatchUnwindFuture::create(fut);
    let res = fut.await;
    match res {
      Ok(result) => {
        tsfn
          .call(result, ThreadsafeFunctionCallMode::NonBlocking)
          .expect("Failed to call JS callback");
      }
      Err(e) => {
        tsfn
          .call(
            Err(Error::from_reason(format!("{e}")).into_napi_error()),
            ThreadsafeFunctionCallMode::NonBlocking,
          )
          .expect("Failed to send panic info");
      }
    }
  });

  Ok(())
}

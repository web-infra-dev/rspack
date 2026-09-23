use std::panic::AssertUnwindSafe;

use futures::{Future, FutureExt};
use rspack_napi::napi::{
  Env, JsError, Result, bindgen_prelude::*, sys, threadsafe_function::ThreadsafeFunctionCallMode,
};

use crate::error::ErrorCode;

/// Run a future, then finalize its result on the JS thread before invoking the
/// error-first callback. The finalizer runs for success, errors and caught panics,
/// and may replace the result (for example, if JS-thread cleanup fails).
pub fn callbackify<R, F>(
  js_callback: Function<'static>,
  fut: F,
  finalizer: Option<impl FnOnce(&Env, &mut Result<R, ErrorCode>) + 'static>,
) -> Result<(), ErrorCode>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R, ErrorCode>>,
{
  let mut finalizer = finalizer;
  let tsfn = js_callback
    // Deliver errors as data: an outer TSFN Err bypasses the conversion callback
    // and would skip finalization before the user's callback.
    .build_threadsafe_function::<Result<R, ErrorCode>>()
    .callee_handled::<false>()
    .max_queue_size::<1>()
    .weak::<false>()
    .build_callback(move |ctx| {
      let mut result = ctx.value;
      if let Some(finalizer) = finalizer.take() {
        finalizer(&ctx.env, &mut result);
      }
      Ok(CallbackArgs(result))
    })
    .map_err(|err| napi::Error::new(ErrorCode::Napi(err.status), err.reason))?;

  rspack_napi::runtime::spawn(async move {
    let res = match AssertUnwindSafe(fut).catch_unwind().await {
      Ok(res) => res,
      Err(payload) => {
        let mut error = rspack_napi::runtime::panic_to_napi_error(payload);
        let reason = std::mem::take(&mut error.reason);
        Err(napi::Error::new(ErrorCode::Napi(error.status), reason))
      }
    };
    tsfn.call(res, ThreadsafeFunctionCallMode::NonBlocking);
  });
  Ok(())
}

/// Preserve error-first arguments and custom error codes without TSFN's outer
/// error handling. Conversion errors must also reach the callback, not become
/// uncaught exceptions under `callee_handled(false)`.
struct CallbackArgs<R>(Result<R, ErrorCode>);

impl<R: ToNapiValue> JsValuesTupleIntoVec for CallbackArgs<R> {
  #[allow(clippy::not_unsafe_ptr_arg_deref)]
  fn into_vec(self, env: sys::napi_env) -> napi::Result<Vec<sys::napi_value>> {
    let result = self.0.and_then(|value| {
      // Match napi-rs argument conversion, including no value argument for ().
      value
        .into_vec(env)
        .map_err(|error| napi::Error::new(ErrorCode::Napi(error.status), error.reason))
    });
    match result {
      Ok(mut args) => {
        // SAFETY: TSFN invokes this conversion in its active JS handle scope.
        args.insert(0, unsafe { Null::to_napi_value(env, Null)? });
        Ok(args)
      }
      Err(error) => {
        // SAFETY: the error is materialized in the same active JS handle scope.
        Ok(vec![unsafe { JsError::from(error).into_value(env) }])
      }
    }
  }
}

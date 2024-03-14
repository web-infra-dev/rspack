use futures::Future;
use rspack_napi::napi::threadsafe_function::{
  ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use rspack_napi::napi::{bindgen_prelude::*, Env, JsFunction, NapiRaw, Result};

// pub fn callbackify<R, F>(env: Env, f: JsFunction, fut: F) -> Result<()>
// where
//   R: 'static + ToNapiValue,
//   F: 'static + Send + Future<Output = Result<R>>,
// {
//   let tsfn = unsafe {
//     ThreadsafeFunction::<R, ErrorStrategy::CalleeHandled>::from_napi_value(env.raw(), f.raw())
//   }?;
//   napi::bindgen_prelude::spawn(async move {
//     let res = fut.await;
//     tsfn.call(res, ThreadsafeFunctionCallMode::NonBlocking);
//   });
//   Ok(())
// }

pub fn callbackify<R, F>(env: Env, f: JsFunction, fut: F) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  let ptr = unsafe { f.raw() };

  let tsfn = rspack_napi::legacy_threadsafe_function::ThreadsafeFunction::<Result<R>, ()>::create(
    env.raw(),
    ptr,
    0,
    |ctx| {
      let rspack_napi::legacy_threadsafe_function::ThreadSafeContext {
        value,
        env,
        callback,
        ..
      } = ctx;

      let argv = match value {
        Ok(value) => {
          let val = unsafe { R::to_napi_value(env.raw(), value)? };
          let js_value = unsafe { rspack_napi::napi::JsUnknown::from_napi_value(env.raw(), val)? };
          vec![env.get_null()?.into_unknown(), js_value]
        }
        Err(err) => {
          vec![JsError::from(err).into_unknown(env)]
        }
      };

      callback.call(None, &argv)?;

      Ok(())
    },
  )?;

  napi::bindgen_prelude::spawn(async move {
    let res = fut.await;
    tsfn
      .call(
        res,
        rspack_napi::legacy_threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
      )
      .expect("Failed to call JS callback");
  });

  Ok(())
}

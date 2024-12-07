use std::sync::Arc;

use futures::Future;
use rspack_napi::napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use rspack_napi::napi::{bindgen_prelude::*, Env, NapiRaw, Result};

pub fn callbackify<R, F>(env: Env, f: Function, fut: F) -> Result<()>
where
  R: 'static + ToNapiValue,
  F: 'static + Send + Future<Output = Result<R>>,
{
  let tsfn = unsafe { ThreadsafeFunction::<R, Unknown>::from_napi_value(env.raw(), f.raw()) }?;
  napi::bindgen_prelude::spawn(async move {
    let res = fut.await;
    tsfn.call(res, ThreadsafeFunctionCallMode::NonBlocking);
  });
  Ok(())
}

pub struct JsArcStr(Arc<str>);

impl JsArcStr {
  pub fn new(val: Arc<str>) -> Self {
    Self(val)
  }
}

impl ToNapiValue for JsArcStr {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let env_wrapper = Env::from(env);
    ToNapiValue::to_napi_value(env, env_wrapper.create_string(val.0.as_ref())?)
  }
}

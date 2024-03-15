use std::thread::{self, ThreadId};

use derivative::Derivative;
use napi::bindgen_prelude::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue};
use napi::{bindgen_prelude::JsValuesTupleIntoVec, Env, JsFunction, Ref};
use napi::{JsUnknown, NapiRaw, NapiValue};

use super::ThreadsafeFunction;
use crate::errors::NapiResultExt;

/// Thread safe function type that
/// - implements `ToNapiValue`, and
/// - can be synchronously called from any thread, including the node thread
#[derive(Derivative)]
#[derivative(Debug)]
#[derivative(Debug(bound = ""))]
pub struct ThreadSafeFunctionWithRef<T: 'static, R> {
  tsfn: ThreadsafeFunction<T, R>,
  // `fn_ref` refers to the same function as `tsfn`.
  // It's used to implement `ToNapiValue` and to get `JsFunction` when called in the node thread.
  #[derivative(Debug = "ignore")]
  fn_ref: Ref<()>,
  thread_id: ThreadId,
}

impl<T: 'static + JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static> ValidateNapiValue
  for ThreadSafeFunctionWithRef<T, R>
{
  unsafe fn validate(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe { JsFunction::validate(env, napi_val) }
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static> TypeName
  for ThreadSafeFunctionWithRef<T, R>
{
  fn type_name() -> &'static str {
    JsFunction::type_name()
  }

  fn value_type() -> napi::ValueType {
    JsFunction::value_type()
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static> FromNapiValue
  for ThreadSafeFunctionWithRef<T, R>
{
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let fn_ref = unsafe {
      Env::from_raw(env).create_reference_with_refcount(
        JsFunction::from_raw(env, napi_val)?,
        // The fn_ref can be weak as tsfn already holds the function.
        // Besides, being weak allows it to be dropped from any thread.
        0,
      )
    }?;
    Ok(Self {
      tsfn: unsafe { ThreadsafeFunction::from_napi_value(env, napi_val) }?,
      fn_ref,
      thread_id: thread::current().id(),
    })
  }
}

impl<T, R> ToNapiValue for ThreadSafeFunctionWithRef<T, R> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    Ok(unsafe {
      Env::from_raw(env)
        .get_reference_value::<JsFunction>(&val.fn_ref)?
        .raw()
    })
  }
}

impl<T: JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static> ThreadSafeFunctionWithRef<T, R> {
  unsafe fn call_in_main(&self, args: T) -> napi::Result<R> {
    let env = unsafe { Env::from_raw(self.tsfn.env) };
    let js_func = env.get_reference_value::<JsFunction>(&self.fn_ref)?;

    let args_raw = args.into_vec(env.raw())?;
    let args = args_raw
      .into_iter()
      .map(|value| unsafe { JsUnknown::from_raw(env.raw(), value) })
      .collect::<napi::Result<Vec<JsUnknown>>>()?;
    let ret = js_func.call(None, &args)?;
    unsafe { R::from_napi_value(env.raw(), ret.raw()) }
  }

  /// Synchronously call the function. Can be called from any thread, including the node thread.
  pub fn call_sync(&self, args: T) -> rspack_error::Result<R> {
    if thread::current().id() != self.thread_id {
      self.tsfn.blocking_call_with_sync(args)
    } else {
      // In the node thread it would cause deadlock if we call tsfn and block until return.
      // So we directly call JsFunction from fn_ref.
      unsafe { self.call_in_main(args).into_rspack_result() }
    }
  }
}

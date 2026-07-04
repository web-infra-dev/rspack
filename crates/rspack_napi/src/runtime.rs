use std::{any::Any, future::Future};

use napi::{
  Env, Error, JsValue, Result, Status,
  bindgen_prelude::{PromiseRaw, ToNapiValue},
};
pub use rspack_tasks::{JoinError, JoinHandle, block_on, ensure_runtime, spawn};

pub fn within_runtime_if_available<F: FnOnce() -> T, T>(f: F) -> T {
  f()
}

pub fn promise_from_future<'env, T, F>(env: &'env Env, future: F) -> Result<PromiseRaw<'env, T>>
where
  T: 'static + Send + ToNapiValue,
  F: 'static + Send + Future<Output = Result<T>>,
{
  let (deferred, promise) = env.create_deferred()?;
  let promise = PromiseRaw::new(env.raw(), promise.raw());
  let deferred_for_panic = deferred.clone();

  let handle = spawn(async move {
    match future.await {
      Ok(value) => deferred.resolve(|_| Ok(value)),
      Err(error) => deferred.reject(error),
    }
  });

  spawn(async move {
    if let Err(error) = handle.await {
      deferred_for_panic.reject(join_error_to_napi_error(error));
    }
  });

  Ok(promise)
}

pub fn panic_to_napi_error(payload: Box<dyn Any + Send + 'static>) -> Error {
  Error::new(Status::GenericFailure, panic_message(payload))
}

fn join_error_to_napi_error(error: JoinError) -> Error {
  if error.is_panic() {
    panic_to_napi_error(error.into_panic())
  } else {
    Error::new(Status::GenericFailure, "Async task was cancelled")
  }
}

fn panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
  if let Some(message) = payload.downcast_ref::<&str>() {
    (*message).to_string()
  } else if let Some(message) = payload.downcast_ref::<String>() {
    message.clone()
  } else {
    "Panic in async function".to_string()
  }
}

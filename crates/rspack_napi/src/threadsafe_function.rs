use std::{
  fmt::Debug,
  marker::PhantomData,
  sync::{Arc, OnceLock},
};

use napi::{
  bindgen_prelude::{FromNapiValue, JsValuesTupleIntoVec, Promise, TypeName, ValidateNapiValue},
  sys::{self, napi_env},
  threadsafe_function::{ThreadsafeFunction as RawThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, JsValue, Status, Unknown, ValueType,
};
use oneshot::Receiver;
use rspack_error::{miette::IntoDiagnostic, Error, Result};

use crate::{JsCallback, NapiErrorToRspackErrorExt};

type ErrorResolver = dyn FnOnce(Env);

static ERROR_RESOLVER: OnceLock<JsCallback<Box<ErrorResolver>>> = OnceLock::new();

pub struct ThreadsafeFunction<T: 'static + JsValuesTupleIntoVec, R> {
  inner: Arc<RawThreadsafeFunction<T, Unknown<'static>, T, Status, false, true>>,
  env: napi_env,
  _data: PhantomData<R>,
}

impl<T: 'static + JsValuesTupleIntoVec, R> Debug for ThreadsafeFunction<T, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ThreadsafeFunction").finish_non_exhaustive()
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for ThreadsafeFunction<T, R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      env: self.env,
      _data: self._data,
    }
  }
}

unsafe impl<T: 'static + JsValuesTupleIntoVec, R> Sync for ThreadsafeFunction<T, R> {}
unsafe impl<T: 'static + JsValuesTupleIntoVec, R> Send for ThreadsafeFunction<T, R> {}

impl<T: 'static + JsValuesTupleIntoVec, R> FromNapiValue for ThreadsafeFunction<T, R> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let inner = unsafe {
      <RawThreadsafeFunction<T, Unknown, T, Status, false, true> as FromNapiValue>::from_napi_value(
        env, napi_val,
      )
    }?;
    let _ = ERROR_RESOLVER
      .get_or_init(|| unsafe { JsCallback::new(env).expect("should initialize error resolver") });
    Ok(Self {
      inner: Arc::new(inner),
      env,
      _data: PhantomData,
    })
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> ThreadsafeFunction<T, R> {
  async fn resolve_error(&self, err: napi::Error) -> Error {
    let (tx, rx) = tokio::sync::oneshot::channel::<rspack_error::Error>();
    ERROR_RESOLVER
      .get()
      // SAFETY: The error resolver is initialized in `FromNapiValue::from_napi_value` and it's the only way to create a tsfn.
      .expect("should have error resolver initialized")
      .call(Box::new(move |env| {
        let err = err.to_rspack_error(&env);
        tx.send(err).expect("failed to resolve js error");
      }));
    rx.await.expect("failed to resolve js error")
  }

  fn call_with_return<D: 'static + FromNapiValue>(&self, value: T) -> Receiver<Result<D>> {
    let (tx, rx) = oneshot::channel::<Result<D>>();
    self
      .inner
      .call_with_return_value(value, ThreadsafeFunctionCallMode::NonBlocking, {
        move |r: napi::Result<Unknown>, env| {
          let r = match r {
            Err(err) => Err(err.to_rspack_error(&env)),
            Ok(o) => unsafe { D::from_napi_value(env.raw(), o.raw()) }.into_diagnostic(),
          };
          tx.send(r)
            .unwrap_or_else(|_| panic!("failed to send tsfn value"));
          Ok(())
        }
      });
    rx
  }

  async fn call_async<D: 'static + FromNapiValue>(&self, value: T) -> Result<D> {
    let rx = self.call_with_return(value);
    rx.await.expect("failed to receive tsfn value")
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue> ThreadsafeFunction<T, R> {
  /// Call the JS function.
  pub async fn call_with_sync(&self, value: T) -> Result<R> {
    self.call_async::<R>(value).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue>
  ThreadsafeFunction<T, Promise<R>>
{
  /// Call the JS function.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, an [napi::Error] is returned.
  pub async fn call_with_promise(&self, value: T) -> Result<R> {
    match self.call_async::<Promise<R>>(value).await {
      Ok(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Err(err) => Err(err),
    }
  }
}

impl<T: 'static + JsValuesTupleIntoVec + JsValuesTupleIntoVec, R> ValidateNapiValue
  for ThreadsafeFunction<T, R>
{
}

impl<T: 'static + JsValuesTupleIntoVec, R> TypeName for ThreadsafeFunction<T, R> {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> napi::ValueType {
    ValueType::Function
  }
}

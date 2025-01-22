use std::{
  fmt::Debug,
  marker::PhantomData,
  sync::{Arc, OnceLock},
};

use napi::{
  bindgen_prelude::{
    Either3, Either4, FromNapiValue, JsValuesTupleIntoVec, Promise, TypeName, ValidateNapiValue,
  },
  sys::{self, napi_env},
  threadsafe_function::{ThreadsafeFunction as RawThreadsafeFunction, ThreadsafeFunctionCallMode},
  Either, Env, JsUnknown as Unknown, NapiRaw, Status, ValueType,
};
use oneshot::Receiver;
use rspack_error::{miette::IntoDiagnostic, Error, Result};

use crate::{JsCallback, NapiErrorExt};

type ErrorResolver = dyn FnOnce(Env);

static ERROR_RESOLVER: OnceLock<JsCallback<Box<ErrorResolver>>> = OnceLock::new();

pub struct ThreadsafeFunction<T: 'static + JsValuesTupleIntoVec, R> {
  inner: Arc<RawThreadsafeFunction<T, Unknown, T, false, true>>,
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
      <RawThreadsafeFunction<T, Unknown, T, false, true> as FromNapiValue>::from_napi_value(
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
        let err = err.into_rspack_error_with_detail(&env);
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
            Err(err) => Err(err.into_rspack_error_with_detail(&env)),
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

  fn blocking_call<D: 'static + FromNapiValue>(&self, value: T) -> Result<D> {
    let rx = self.call_with_return(value);
    rx.recv().expect("failed to receive tsfn value")
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> ThreadsafeFunction<T, R> {
  /// Synchronously call JS function and report error as `uncaughtException`.
  /// See: [napi_create_threadsafe_function](https://nodejs.org/dist/latest/docs/api/n-api.html#napi_create_threadsafe_function)
  pub fn call_with_fatal(&self, value: T) {
    let status = self
      .inner
      .call(value, ThreadsafeFunctionCallMode::NonBlocking);
    debug_assert_eq!(status, Status::Ok, "failed to call tsfn with fatal")
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue> ThreadsafeFunction<T, R> {
  /// Call the JS function.
  pub async fn call_with_sync(&self, value: T) -> Result<R> {
    self.call_async::<R>(value).await
  }

  /// Call the JS function with blocking.
  pub fn blocking_call_with_sync(&self, value: T) -> Result<R> {
    self.blocking_call::<R>(value)
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: 'static + FromNapiValue + ValidateNapiValue>
  ThreadsafeFunction<T, R>
{
  /// Call the JS function.
  /// This method expects the returned value of JS function to be a `Promise<R>` or `R`.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  ///
  /// ## Warning
  /// This method is **NOT** recommended to be used in most cases. It makes return value ambiguous.
  pub async fn call(&self, value: T) -> Result<R> {
    match self.call_async::<Either<Promise<R>, R>>(value).await {
      Ok(Either::A(r)) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Ok(Either::B(r)) => Ok(r),
      Err(err) => Err(err),
    }
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

impl<
    T: 'static + JsValuesTupleIntoVec,
    R: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either<Promise<R>, R>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<R> {
    match self.call_async::<Either<Promise<R>, R>>(value).await? {
      Either::A(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Either::B(r) => Ok(r),
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    R: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either<R, Promise<R>>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<R> {
    match self.call_async::<Either<R, Promise<R>>>(value).await? {
      Either::A(r) => Ok(r),
      Either::B(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either3<T0, T1, Promise<Either<T0, T1>>>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either<T0, T1>> {
    match self
      .call_async::<Either3<T0, T1, Promise<Either<T0, T1>>>>(value)
      .await?
    {
      Either3::A(r) => Ok(Either::A(r)),
      Either3::B(r) => Ok(Either::B(r)),
      Either3::C(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either3<T0, Promise<Either<T0, T1>>, T1>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either<T0, T1>> {
    match self
      .call_async::<Either3<T0, Promise<Either<T0, T1>>, T1>>(value)
      .await?
    {
      Either3::A(r) => Ok(Either::A(r)),
      Either3::B(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Either3::C(r) => Ok(Either::B(r)),
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either3<Promise<Either<T0, T1>>, T0, T1>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either<T0, T1>> {
    match self
      .call_async::<Either3<Promise<Either<T0, T1>>, T0, T1>>(value)
      .await?
    {
      Either3::A(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Either3::B(r) => Ok(Either::A(r)),
      Either3::C(r) => Ok(Either::B(r)),
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T2: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either4<T0, T1, T2, Promise<Either3<T0, T1, T2>>>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either3<T0, T1, T2>> {
    match self
      .call_async::<Either4<T0, T1, T2, Promise<Either3<T0, T1, T2>>>>(value)
      .await?
    {
      Either4::A(r) => Ok(Either3::A(r)),
      Either4::B(r) => Ok(Either3::B(r)),
      Either4::C(r) => Ok(Either3::C(r)),
      Either4::D(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T2: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either4<T0, T1, Promise<Either3<T0, T1, T2>>, T2>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either3<T0, T1, T2>> {
    match self
      .call_async::<Either4<T0, T1, Promise<Either3<T0, T1, T2>>, T2>>(value)
      .await?
    {
      Either4::A(r) => Ok(Either3::A(r)),
      Either4::B(r) => Ok(Either3::B(r)),
      Either4::C(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Either4::D(r) => Ok(Either3::C(r)),
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T2: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either4<T0, Promise<Either3<T0, T1, T2>>, T1, T2>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either3<T0, T1, T2>> {
    match self
      .call_async::<Either4<T0, Promise<Either3<T0, T1, T2>>, T1, T2>>(value)
      .await?
    {
      Either4::A(r) => Ok(Either3::A(r)),
      Either4::B(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Either4::C(r) => Ok(Either3::B(r)),
      Either4::D(r) => Ok(Either3::C(r)),
    }
  }
}

impl<
    T: 'static + JsValuesTupleIntoVec,
    T0: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T1: 'static + FromNapiValue + ValidateNapiValue + TypeName,
    T2: 'static + FromNapiValue + ValidateNapiValue + TypeName,
  > ThreadsafeFunction<T, Either4<Promise<Either3<T0, T1, T2>>, T0, T1, T2>>
{
  /// Call the JS function and resolve the returned value depending on its type.
  /// If `Promise<T>` is returned, it will be awaited and its value `T` will be returned.
  /// Otherwise, if `T` is returned, it will be returned as-is.
  pub async fn call_with_auto(&self, value: T) -> Result<Either3<T0, T1, T2>> {
    match self
      .call_async::<Either4<Promise<Either3<T0, T1, T2>>, T0, T1, T2>>(value)
      .await?
    {
      Either4::A(r) => match r.await {
        Ok(r) => Ok(r),
        Err(err) => Err(self.resolve_error(err).await),
      },
      Either4::B(r) => Ok(Either3::A(r)),
      Either4::C(r) => Ok(Either3::B(r)),
      Either4::D(r) => Ok(Either3::C(r)),
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

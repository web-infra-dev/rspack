use std::{
  any::Any,
  cell::Cell,
  fmt::Debug,
  marker::PhantomData,
  rc::Rc,
  sync::{Arc, OnceLock},
};

use napi::{
  Env, JsValue, Status, Unknown, ValueType,
  bindgen_prelude::{
    CallbackContext, FromNapiValue, JsValuesTupleIntoVec, Promise, PromiseRaw, TypeName,
    ValidateNapiValue,
  },
  sys::{self, napi_env, napi_value},
  threadsafe_function::{ThreadsafeFunction as RawThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use oneshot::{Receiver, channel};
use rspack_error::{Error, Result};

use crate::{JsCallback, NapiErrorToRspackErrorExt};

type ErrorResolver = dyn FnOnce(Env);

static ERROR_RESOLVER: OnceLock<JsCallback<Box<ErrorResolver>>> = OnceLock::new();

pub struct ThreadsafeFunction<T: 'static + JsValuesTupleIntoVec, R> {
  inner: Arc<RawThreadsafeFunction<T, Unknown<'static>, T, Status, false, true>>,
  env: napi_env,
  _data: PhantomData<R>,
}

pub struct DynThreadsafeFunction {
  inner: Arc<RawThreadsafeFunction<DynJsArgs, Unknown<'static>, DynJsArgs, Status, false, true>>,
}

struct DynJsArgs(Box<dyn DynJsArgsToVec>);

trait DynJsArgsToVec {
  fn into_vec(self: Box<Self>, env: napi_env) -> napi::Result<Vec<napi_value>>;
}

impl<T: JsValuesTupleIntoVec> DynJsArgsToVec for T {
  fn into_vec(self: Box<Self>, env: napi_env) -> napi::Result<Vec<napi_value>> {
    (*self).into_vec(env)
  }
}

impl JsValuesTupleIntoVec for DynJsArgs {
  fn into_vec(self, env: napi_env) -> napi::Result<Vec<napi_value>> {
    self.0.into_vec(env)
  }
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

impl Debug for DynThreadsafeFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DynThreadsafeFunction")
      .finish_non_exhaustive()
  }
}

impl Clone for DynThreadsafeFunction {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

unsafe impl<T: 'static + JsValuesTupleIntoVec, R> Sync for ThreadsafeFunction<T, R> {}
unsafe impl<T: 'static + JsValuesTupleIntoVec, R> Send for ThreadsafeFunction<T, R> {}
unsafe impl Sync for DynThreadsafeFunction {}
unsafe impl Send for DynThreadsafeFunction {}

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

impl FromNapiValue for DynThreadsafeFunction {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let inner = unsafe {
      <RawThreadsafeFunction<DynJsArgs, Unknown, DynJsArgs, Status, false, true> as FromNapiValue>::from_napi_value(
        env, napi_val,
      )
    }?;
    let _ = ERROR_RESOLVER
      .get_or_init(|| unsafe { JsCallback::new(env).expect("should initialize error resolver") });
    Ok(Self {
      inner: Arc::new(inner),
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
    let (tx, rx) = channel::<Result<D>>();
    self
      .inner
      .call_with_return_value(value, ThreadsafeFunctionCallMode::NonBlocking, {
        move |r: napi::Result<Unknown>, env| {
          let r = match r {
            Err(err) => Err(err.to_rspack_error(&env)),
            Ok(o) => {
              let raw_env = env.raw();
              let return_value = o.raw();
              unsafe { D::from_napi_value(raw_env, return_value) }
                .map_err(|e| pretty_type_error(o, e))
            }
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

// Only converted Rust values cross back to the caller. The decoder always runs on
// the JS thread; no napi_value or Unknown is stored in the completion channel.
type DynJsValue = Box<dyn Any + Send>;
type DynJsDecoder = fn(Env, Unknown<'_>) -> napi::Result<DynJsValue>;
type DynJsCompletion = Rc<Cell<Option<oneshot::Sender<Result<DynJsValue>>>>>;

fn decode_js_value<R: 'static + FromNapiValue + Send>(
  env: Env,
  value: Unknown<'_>,
) -> napi::Result<DynJsValue> {
  // SAFETY: The decoder is invoked by the TSFN or Promise callback on the JS thread.
  unsafe { R::from_napi_value(env.raw(), value.raw()) }.map(|value| Box::new(value) as DynJsValue)
}

fn take_js_value<R: 'static + Send>(value: DynJsValue) -> R {
  *value
    .downcast::<R>()
    .expect("TSFN decoder must match its caller's return type")
}

fn complete_js_call(completion: &DynJsCompletion, result: Result<DynJsValue>) {
  if let Some(sender) = completion.take() {
    // A caller may drop its future while a Promise is still pending.
    let _ = sender.send(result);
  }
}

fn resolve_js_promise(
  env: Env,
  value: Unknown<'_>,
  decode: DynJsDecoder,
  sender: oneshot::Sender<Result<DynJsValue>>,
) {
  let completion = Rc::new(Cell::new(Some(sender)));
  let on_resolve = completion.clone();
  let on_reject = completion.clone();
  // Use one PromiseRaw<Unknown> implementation for every hook return type. Both
  // conversion and rejection handling stay on the JS thread.
  // SAFETY: `value` belongs to the active TSFN callback's environment.
  let result = unsafe { PromiseRaw::<Unknown>::from_napi_value(env.raw(), value.raw()) }
    .and_then(|promise| {
      promise.then(move |ctx| {
        // Propagate conversion errors through the chained catch, just like
        // napi::Promise<R>, to preserve JS error conversion and stack handling.
        let value = decode(ctx.env, ctx.value)?;
        complete_js_call(&on_resolve, Ok(value));
        Ok(())
      })
    })
    .and_then(|promise| {
      promise.catch(move |ctx: CallbackContext<Unknown>| {
        let err = napi::Error::from(ctx.value).to_rspack_error(&ctx.env);
        complete_js_call(&on_reject, Err(err));
        Ok(())
      })
    });
  if let Err(err) = result {
    complete_js_call(&completion, Err(pretty_type_error(value, err)));
  }
}

impl DynThreadsafeFunction {
  fn call_with_return(
    &self,
    value: DynJsArgs,
    decode: DynJsDecoder,
    promise: bool,
  ) -> Receiver<Result<DynJsValue>> {
    let (tx, rx) = channel();
    self.inner.call_with_return_value(
      value,
      ThreadsafeFunctionCallMode::NonBlocking,
      move |result: napi::Result<Unknown>, env| {
        let result = match result {
          Err(err) => Err(err.to_rspack_error(&env)),
          Ok(value) if promise => {
            resolve_js_promise(env, value, decode, tx);
            return Ok(());
          }
          Ok(value) => decode(env, value).map_err(|err| pretty_type_error(value, err)),
        };
        tx.send(result)
          .unwrap_or_else(|_| panic!("failed to send tsfn value"));
        Ok(())
      },
    );
    rx
  }

  // Dispatch before constructing the future so it only owns the Send completion
  // receiver, not the JS-thread argument conversion payload.
  fn call_async(
    &self,
    value: DynJsArgs,
    decode: DynJsDecoder,
    promise: bool,
  ) -> impl std::future::Future<Output = Result<DynJsValue>> + Send + use<> {
    let rx = self.call_with_return(value, decode, promise);
    async move { rx.await.expect("failed to receive tsfn value") }
  }

  /// Call the JS function.
  pub async fn call_with_sync<T, R>(&self, value: T) -> Result<R>
  where
    T: 'static + JsValuesTupleIntoVec,
    R: 'static + FromNapiValue + Send,
  {
    self
      .call_async(DynJsArgs(Box::new(value)), decode_js_value::<R>, false)
      .await
      .map(take_js_value::<R>)
  }

  /// Call the JS function and await its returned Promise.
  pub async fn call_with_promise<T, R>(&self, value: T) -> Result<R>
  where
    T: 'static + JsValuesTupleIntoVec,
    R: 'static + FromNapiValue + Send,
  {
    self
      .call_async(DynJsArgs(Box::new(value)), decode_js_value::<R>, true)
      .await
      .map(take_js_value::<R>)
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

impl ValidateNapiValue for DynThreadsafeFunction {}

impl<T: 'static + JsValuesTupleIntoVec, R> TypeName for ThreadsafeFunction<T, R> {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> napi::ValueType {
    ValueType::Function
  }
}

impl TypeName for DynThreadsafeFunction {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> napi::ValueType {
    ValueType::Function
  }
}

fn pretty_type_error(return_value: Unknown, error: napi::Error) -> rspack_error::Error {
  let expected_type = match error.status {
    Status::ObjectExpected => "object",
    Status::StringExpected => "string",
    Status::FunctionExpected => "function",
    Status::NumberExpected => "number",
    Status::BooleanExpected => "boolean",
    Status::ArrayExpected => "Array",
    Status::BigintExpected => "bigint",
    Status::DateExpected => "Date",
    Status::ArrayBufferExpected => "ArrayBuffer",
    _ => return rspack_error::error!("{}", error),
  };
  let reason = match return_value.get_type() {
    Ok(return_value_type) => {
      let return_value_type_str = match return_value_type {
        ValueType::Undefined => "undefined",
        ValueType::Null => "null",
        ValueType::Boolean => "boolean",
        ValueType::Number => "number",
        ValueType::String => "string",
        ValueType::Symbol => "symbol",
        ValueType::Object => "object",
        ValueType::Function => "function",
        ValueType::External => "external",
        ValueType::BigInt => "bigint",
        _ => "unknown",
      };
      format!(
        "TypeError: Expected return a '{expected_type}' value, but received `{return_value_type_str}`"
      )
    }
    Err(_) => format!("TypeError: Expected return a '{expected_type}' value"),
  };
  rspack_error::error!(reason)
}

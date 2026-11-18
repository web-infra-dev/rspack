use std::{
  cell::Cell,
  ffi::c_void,
  fmt::{self, Debug},
  marker::PhantomData,
  ptr,
  rc::Rc,
  sync::{Arc, Weak},
};

use napi::{
  Env, JsValue, Status, Unknown, ValueType,
  bindgen_prelude::{
    CallbackContext, FromNapiValue, JsValuesTupleIntoVec, Promise, PromiseRaw, TypeName,
    ValidateNapiValue,
  },
  check_status,
  sys::{self, napi_env, napi_value},
  threadsafe_function::ThreadsafeFunctionHandle,
};
use oneshot::{Sender, channel};
use rspack_error::Result;

use crate::NapiErrorToRspackErrorExt;

type JsCall = Box<dyn FnOnce(Env, napi_value) + Send>;

pub struct ThreadsafeFunction<T: 'static + JsValuesTupleIntoVec, R> {
  inner: DynThreadsafeFunction,
  _types: PhantomData<fn(T) -> R>,
}

#[derive(Clone)]
pub struct DynThreadsafeFunction {
  inner: Arc<ThreadsafeFunctionHandle>,
}

impl<T: 'static + JsValuesTupleIntoVec, R> Debug for ThreadsafeFunction<T, R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ThreadsafeFunction").finish_non_exhaustive()
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> Clone for ThreadsafeFunction<T, R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      _types: PhantomData,
    }
  }
}

impl Debug for DynThreadsafeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynThreadsafeFunction")
      .finish_non_exhaustive()
  }
}

unsafe extern "C" fn finalize_tsfn<F: FnOnce()>(
  _env: napi_env,
  data: *mut c_void,
  _hint: *mut c_void,
) {
  let (handle, finalize) =
    *unsafe { Box::<(Weak<ThreadsafeFunctionHandle>, F)>::from_raw(data.cast()) };
  if let Some(handle) = handle.upgrade() {
    handle.with_write_aborted(|mut aborted| *aborted = true);
  }
  finalize();
}

unsafe extern "C" fn run_js_call(
  env: napi_env,
  callback: napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  let call = *unsafe { Box::<JsCall>::from_raw(data.cast()) };
  if !env.is_null() && !callback.is_null() {
    call(Env::from_raw(env), callback);
  }
}

impl DynThreadsafeFunction {
  fn new<F: FnOnce() + 'static>(
    env: napi_env,
    callback: napi_value,
    finalize: F,
  ) -> napi::Result<Self> {
    let finalize = scopeguard::guard(finalize, |finalize| finalize());
    let mut name = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_string_utf8(env, c"rspack:callback".as_ptr(), 15, &mut name)
    })?;
    let inner = ThreadsafeFunctionHandle::null();
    let data = Box::into_raw(Box::new((
      Arc::downgrade(&inner),
      scopeguard::ScopeGuard::into_inner(finalize),
    )));
    let mut raw = ptr::null_mut();
    let status = unsafe {
      sys::napi_create_threadsafe_function(
        env,
        callback,
        ptr::null_mut(),
        name,
        0,
        1,
        data.cast(),
        Some(finalize_tsfn::<F>),
        ptr::null_mut(),
        Some(run_js_call),
        &mut raw,
      )
    };
    if status != sys::Status::napi_ok {
      unsafe { finalize_tsfn::<F>(env, data.cast(), ptr::null_mut()) };
      check_status!(status)?;
    }
    inner.set_raw(raw);
    check_status!(unsafe { sys::napi_unref_threadsafe_function(env, raw) })?;
    Ok(Self { inner })
  }

  fn post(&self, call: JsCall) -> napi::Result<()> {
    let data = Box::into_raw(Box::new(call));
    let status = self.inner.with_write_aborted(|mut aborted| {
      if *aborted {
        return sys::Status::napi_closing;
      }
      let status = unsafe {
        sys::napi_call_threadsafe_function(
          self.inner.get_raw(),
          data.cast(),
          sys::ThreadsafeFunctionCallMode::nonblocking,
        )
      };
      if status == sys::Status::napi_closing {
        *aborted = true;
      }
      status
    });
    if status != sys::Status::napi_ok {
      drop(unsafe { Box::from_raw(data) });
    }
    check_status!(status, "Unable to deliver JavaScript callback")
  }

  async fn dispatch<R: Send + 'static>(
    &self,
    call: impl FnOnce(&Env, napi_value, Sender<Result<R>>) + Send + 'static,
  ) -> Result<R> {
    let (tx, rx) = channel();
    self
      .post(Box::new(move |env, callback| call(&env, callback, tx)))
      .map_err(|error| rspack_error::error!("{error}"))?;
    rx.await
      .map_err(|_| rspack_error::error!("JavaScript environment closed before the callback ran"))?
  }

  /// Prepare the arguments and their guard in the owning JS environment. Drop
  /// the guard when the function returns, before converting its result.
  pub async fn call_with_sync_prepared<R, Guard>(
    &self,
    prepare: impl FnOnce(&Env) -> napi::Result<(Vec<napi_value>, Guard)> + Send + 'static,
  ) -> Result<R>
  where
    R: FromNapiValue + Send + 'static,
  {
    self
      .dispatch(move |env, callback, done| {
        let result = invoke_prepared(env, callback, prepare).and_then(|(value, guard)| {
          drop(guard);
          convert_result(env, value)
        });
        let _ = done.send(result);
      })
      .await
  }

  pub async fn call_with_sync<T, R>(&self, value: T) -> Result<R>
  where
    T: JsValuesTupleIntoVec + Send + 'static,
    R: FromNapiValue + Send + 'static,
  {
    self
      .call_with_sync_prepared(move |env| Ok((value.into_vec(env.raw())?, ())))
      .await
  }

  pub async fn call_with_promise<T, R>(&self, value: T) -> Result<R>
  where
    T: JsValuesTupleIntoVec + Send + 'static,
    R: FromNapiValue + Send + 'static,
  {
    self
      .call_with_promise_prepared(move |env| Ok((value.into_vec(env.raw())?, ())))
      .await
  }

  async fn call_with_promise_prepared<R, Guard>(
    &self,
    prepare: impl FnOnce(&Env) -> napi::Result<(Vec<napi_value>, Guard)> + Send + 'static,
  ) -> Result<R>
  where
    R: FromNapiValue + Send + 'static,
    Guard: 'static,
  {
    self
      .dispatch(
        move |env, callback, done| match invoke_prepared(env, callback, prepare) {
          Ok((value, guard)) => complete_promise(env, value, guard, done),
          Err(error) => {
            let _ = done.send(Err(error));
          }
        },
      )
      .await
  }
}

fn invoke_prepared<'env, Guard>(
  env: &'env Env,
  callback: napi_value,
  prepare: impl FnOnce(&Env) -> napi::Result<(Vec<napi_value>, Guard)>,
) -> Result<(Unknown<'env>, Guard)> {
  let (args, guard) = prepare(env).map_err(|error| js_error(env, error))?;
  match call_js_function(env, callback, &args) {
    Ok(value) => Ok((value, guard)),
    Err(error) => {
      drop(guard);
      Err(js_error(env, error))
    }
  }
}

fn complete_promise<R: FromNapiValue + 'static, Guard: 'static>(
  env: &Env,
  value: Unknown<'_>,
  guard: Guard,
  done: Sender<Result<R>>,
) {
  // The JS reactions own the guard and the sender together. Every completion
  // path revokes direct access before letting the Rust caller resume.
  let pending = Rc::new(Cell::new(Some((guard, done))));
  let fulfilled = Rc::clone(&pending);
  let rejected = Rc::clone(&pending);
  let registered = PromiseRaw::new(env.raw(), value.raw())
    .then(move |ctx: CallbackContext<Unknown>| {
      if let Some((guard, done)) = fulfilled.take() {
        drop(guard);
        let _ = done.send(convert_result(&ctx.env, ctx.value));
      }
      Ok(())
    })
    .and_then(|promise| {
      promise.catch(move |ctx: CallbackContext<Unknown>| {
        if let Some((guard, done)) = rejected.take() {
          drop(guard);
          let error = napi::Error::from_unknown_without_coercion(ctx.value);
          let _ = done.send(Err(error.to_rspack_error(&ctx.env)));
        }
        Ok(())
      })
    });
  if let Err(error) = registered {
    let done = pending.take().map(|(guard, done)| {
      drop(guard);
      done
    });
    let error = js_error(env, error);
    if let Some(done) = done {
      let _ = done.send(Err(error));
    }
  }
}

fn call_js_function<'env>(
  env: &'env Env,
  callback: napi_value,
  args: &[napi_value],
) -> napi::Result<Unknown<'env>> {
  let mut receiver = ptr::null_mut();
  check_status!(unsafe { sys::napi_get_undefined(env.raw(), &mut receiver) })?;
  let mut result = ptr::null_mut();
  check_status!(unsafe {
    sys::napi_call_function(
      env.raw(),
      receiver,
      callback,
      args.len(),
      args.as_ptr(),
      &mut result,
    )
  })?;
  Ok(unsafe { Unknown::from_raw_unchecked(env.raw(), result) })
}

fn js_error(env: &Env, error: napi::Error) -> rspack_error::Error {
  let mut pending = false;
  if error.status == Status::PendingException
    && unsafe { sys::napi_is_exception_pending(env.raw(), &mut pending) } == sys::Status::napi_ok
    && pending
  {
    let mut value = ptr::null_mut();
    if unsafe { sys::napi_get_and_clear_last_exception(env.raw(), &mut value) }
      == sys::Status::napi_ok
    {
      return napi::Error::from_unknown_without_coercion(unsafe {
        Unknown::from_raw_unchecked(env.raw(), value)
      })
      .to_rspack_error(env);
    }
  }
  error.to_rspack_error(env)
}

fn convert_result<R: FromNapiValue>(env: &Env, value: Unknown<'_>) -> Result<R> {
  unsafe { R::from_napi_value(env.raw(), value.raw()) }.map_err(|error| {
    if error.status == Status::PendingException {
      js_error(env, error)
    } else {
      pretty_type_error(value, error)
    }
  })
}

impl<T: 'static + JsValuesTupleIntoVec, R> FromNapiValue for ThreadsafeFunction<T, R> {
  unsafe fn from_napi_value(env: napi_env, callback: napi_value) -> napi::Result<Self> {
    unsafe { Self::from_napi_value_with_finalize(env, callback, || {}) }
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> ThreadsafeFunction<T, R> {
  /// Runs `finalize` on the owning JS thread when N-API finalizes the TSFN, or
  /// during construction if it fails. The callback must not retain the TSFN or panic.
  ///
  /// # Safety
  /// `env` and `callback` must be valid on the calling JS thread.
  pub unsafe fn from_napi_value_with_finalize(
    env: napi_env,
    callback: napi_value,
    finalize: impl FnOnce() + 'static,
  ) -> napi::Result<Self> {
    Ok(Self {
      inner: DynThreadsafeFunction::new(env, callback, finalize)?,
      _types: PhantomData,
    })
  }
}

impl FromNapiValue for DynThreadsafeFunction {
  unsafe fn from_napi_value(env: napi_env, callback: napi_value) -> napi::Result<Self> {
    Self::new(env, callback, || {})
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static>
  ThreadsafeFunction<T, R>
{
  pub async fn call_with_sync(&self, value: T) -> Result<R>
  where
    T: Send,
  {
    self.inner.call_with_sync(value).await
  }

  pub async fn call_with_sync_prepared<Guard>(
    &self,
    prepare: impl FnOnce(&Env) -> napi::Result<(Vec<napi_value>, Guard)> + Send + 'static,
  ) -> Result<R> {
    self.inner.call_with_sync_prepared(prepare).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R: FromNapiValue + Send + 'static>
  ThreadsafeFunction<T, Promise<R>>
{
  pub async fn call_with_promise(&self, value: T) -> Result<R>
  where
    T: Send,
  {
    self.inner.call_with_promise(value).await
  }

  /// Holds the JS-thread guard until the Promise fulfills or rejects.
  pub async fn call_with_promise_prepared<Guard: 'static>(
    &self,
    prepare: impl FnOnce(&Env) -> napi::Result<(Vec<napi_value>, Guard)> + Send + 'static,
  ) -> Result<R> {
    self.inner.call_with_promise_prepared(prepare).await
  }
}

impl<T: 'static + JsValuesTupleIntoVec, R> ValidateNapiValue for ThreadsafeFunction<T, R> {}

impl ValidateNapiValue for DynThreadsafeFunction {}

impl<T: 'static + JsValuesTupleIntoVec, R> TypeName for ThreadsafeFunction<T, R> {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> ValueType {
    ValueType::Function
  }
}

impl TypeName for DynThreadsafeFunction {
  fn type_name() -> &'static str {
    "ThreadsafeFunction"
  }

  fn value_type() -> ValueType {
    ValueType::Function
  }
}

fn pretty_type_error(return_value: Unknown, error: napi::Error) -> rspack_error::Error {
  let expected_type = match error.status {
    Status::ObjectExpected => "object",
    Status::StringExpected => "string",
    Status::FunctionExpected => "function",
    Status::BooleanExpected => "boolean",
    Status::NumberExpected => "number",
    Status::BigintExpected => "bigint",
    Status::ArrayExpected => "array",
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
        ValueType::Unknown => "unknown",
      };
      format!(
        "TypeError: Expected return a '{expected_type}' value, but received `{return_value_type_str}`"
      )
    }
    Err(_) => format!("TypeError: Expected return a '{expected_type}' value"),
  };
  rspack_error::error!(reason)
}

#[cfg(allocative)]
impl<T: 'static + JsValuesTupleIntoVec, R> rspack_allocative::Allocative
  for ThreadsafeFunction<T, R>
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut rspack_allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(rspack_allocative::Key::new("inner"), &self.inner);
    visitor.exit();
  }
}

#[cfg(allocative)]
impl rspack_allocative::Allocative for DynThreadsafeFunction {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut rspack_allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(rspack_allocative::Key::new("inner"), &self.inner);
    visitor.exit();
  }
}

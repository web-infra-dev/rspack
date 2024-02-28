// Modified based on https://github.com/parcel-bundler/lightningcss/blob/865270134da6f25d66ce4a4ec0e4c4b93b90b759/node/src/threadsafe_function.rs
// Fork of threadsafe_function from napi-rs that allows calling JS function manually rather than
// only returning args. This enables us to use the return value of the function.

#![allow(clippy::single_component_path_imports)]
#![allow(unused)]

use std::convert::Into;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use napi::bindgen_prelude::{FromNapiValue, Promise, ToNapiValue};
use napi::{check_status, sys, Env, JsUnknown, NapiRaw, Result, Status};
use napi::{JsError, JsFunction, NapiValue};
use rspack_error::{error, InternalError};

use super::{NapiErrorExt, NapiResultExt};

/// ThreadSafeFunction Context object
/// the `value` is the value passed to `call` method
pub struct ThreadSafeContext<T: 'static, R> {
  pub env: Env,
  pub value: T,
  pub callback: JsFunction,
  pub tx: tokio::sync::oneshot::Sender<rspack_error::Result<R>>,
}

pub struct ThreadSafeCallContext<T: 'static> {
  pub env: Env,
  pub value: T,
  pub callback: JsFunction,
}

pub struct ThreadSafeResolver<R> {
  pub env: Env,
  pub tx: tokio::sync::oneshot::Sender<rspack_error::Result<R>>,
}

impl<T: 'static, R> ThreadSafeContext<T, R> {
  /// Split context into two parts, good for calling JS function, and resolving the return value in separate steps
  pub fn split_into_parts(self) -> (ThreadSafeCallContext<T>, ThreadSafeResolver<R>) {
    let call_context = ThreadSafeCallContext {
      env: self.env,
      value: self.value,
      callback: self.callback,
    };
    let resolver = ThreadSafeResolver {
      env: self.env,
      tx: self.tx,
    };

    (call_context, resolver)
  }
}

impl<R: 'static + Send> ThreadSafeResolver<R> {
  /// Consume the context and resolve the result from Node side. Calling the real callback should be happened before this.
  ///
  /// Since the original calling of threadsafe function is a pure enqueue operation,
  /// no matter a plain data structure or a `Promise` is returned, we need to send the message to the receiver side.
  ///
  /// Note:
  /// Return an recoverable Rust error is not preferred as it will become a fatal error on the Node side. See `call_js_cb` for more details.
  /// Often, the result of the real call-in-js operation is passed as the `result`.
  pub fn resolve<P>(
    mut self,
    result: Result<impl NapiRaw>,
    resolver: impl 'static + Send + Sync + FnOnce(&mut Env, P) -> Result<R>,
  ) -> Result<()>
  where
    // Pure return value without promise wrapper
    P: FromNapiValue + Send + 'static,
  {
    match result {
      Ok(result) => {
        let raw = unsafe { result.raw() };

        let mut is_promise = false;
        check_status!(unsafe { sys::napi_is_promise(self.env.raw(), raw, &mut is_promise) })?;

        if is_promise {
          let p = unsafe { Promise::<P>::from_napi_value(self.env.raw(), raw) }?;

          self
            .env
            .execute_tokio_future(async move { Ok(p.await) }, |env, p| {
              self
                .tx
                .send(
                  p.and_then(|r| resolver(env, r))
                    .into_rspack_result_with_detail(env),
                )
                .map_err(|_| napi::Error::from_reason("Failed to send resolved value".to_owned()))
            })?;

          return Ok(());
        }

        let p = {
          let p = unsafe { P::from_napi_value(self.env.raw(), raw) }?;
          resolver(&mut self.env, p)
        };

        self
          .tx
          .send(p.into_rspack_result_with_detail(&self.env))
          .map_err(|_| napi::Error::from_reason("Failed to send resolve message".to_string()))
      }
      Err(e) => self
        .tx
        .send(Err(e.into_rspack_error_with_detail(&self.env)))
        .map_err(|_| napi::Error::from_reason("Failed to send resolve message".to_string())),
    }
  }
}

impl<R: 'static> ThreadSafeResolver<R> {
  pub fn resolve_non_promise<P>(
    mut self,
    result: Result<impl NapiRaw>,
    resolver: impl 'static + Send + Sync + FnOnce(&mut Env, P) -> Result<R>,
  ) -> Result<()>
  where
    P: FromNapiValue + 'static,
  {
    match result {
      Ok(result) => {
        let raw = unsafe { result.raw() };

        debug_assert!(
          {
            let mut is_promise = false;
            check_status!(unsafe { sys::napi_is_promise(self.env.raw(), raw, &mut is_promise) })?;
            !is_promise
          },
          "The result of the ThreadsafeFunction should not be a Promise"
        );

        let p = {
          let p = unsafe { P::from_napi_value(self.env.raw(), raw) }?;
          resolver(&mut self.env, p)
        };

        self
          .tx
          .send(p.into_rspack_result_with_detail(&self.env))
          .map_err(|_| napi::Error::from_reason("Failed to send resolve message".to_string()))
      }
      Err(e) => self
        .tx
        .send(Err(e.into_rspack_error_with_detail(&self.env)))
        .map_err(|_| napi::Error::from_reason("Failed to send resolve message".to_string())),
    }
  }
}

#[repr(u8)]
pub enum ThreadsafeFunctionCallMode {
  NonBlocking,
  Blocking,
}

impl From<ThreadsafeFunctionCallMode> for sys::napi_threadsafe_function_call_mode {
  fn from(value: ThreadsafeFunctionCallMode) -> Self {
    match value {
      ThreadsafeFunctionCallMode::Blocking => sys::ThreadsafeFunctionCallMode::blocking,
      ThreadsafeFunctionCallMode::NonBlocking => sys::ThreadsafeFunctionCallMode::nonblocking,
    }
  }
}

/// Communicate with the addon's main thread by invoking a JavaScript function from other threads.
///
/// ## Example
/// An example of using `ThreadsafeFunction`:
///
/// ```rust,ignore
/// #[macro_use]
/// extern crate napi_derive;
///
/// use std::thread;
///
/// use napi::{
///   threadsafe_function::{
///     ThreadSafeContext, ThreadsafeFunctionCallMode, ThreadsafeFunctionReleaseMode,
///   },
///   CallContext, Error, JsFunction, JsNumber, JsUndefined, Result, Status,
/// };
///
/// #[js_function(1)]
/// pub fn test_threadsafe_function(ctx: CallContext) -> Result<JsUndefined> {
///   let func = ctx.get::<JsFunction>(0)?;
///
///   let tsfn =
///     ctx
///       .env
///       .create_threadsafe_function(&func, 0, |ctx: ThreadSafeContext<Vec<u32>>| {
///         ctx
///           .value
///           .iter()
///           .map(|v| ctx.env.create_uint32(*v))
///           .collect::<Result<Vec<JsNumber>>>()
///       })?;
///
///   let tsfn_cloned = tsfn.clone();
///
///   thread::spawn(move || {
///     let output: Vec<u32> = vec![0, 1, 2, 3];
///     // It's okay to call a threadsafe function multiple times.
///     tsfn.call(Ok(output.clone()), ThreadsafeFunctionCallMode::Blocking);
///   });
///
///   thread::spawn(move || {
///     let output: Vec<u32> = vec![3, 2, 1, 0];
///     // It's okay to call a threadsafe function multiple times.
///     tsfn_cloned.call(Ok(output.clone()), ThreadsafeFunctionCallMode::NonBlocking);
///   });
///
///   ctx.env.get_undefined()
/// }
/// ```
pub struct ThreadsafeFunction<T: 'static, R> {
  raw_tsfn: sys::napi_threadsafe_function,
  aborted: Arc<AtomicBool>,
  ref_count: Arc<AtomicUsize>,
  _phantom: PhantomData<(T, R)>,
}

impl<T: 'static, R> Clone for ThreadsafeFunction<T, R> {
  fn clone(&self) -> Self {
    if !self.aborted.load(Ordering::Acquire) {
      let acquire_status = unsafe { sys::napi_acquire_threadsafe_function(self.raw_tsfn) };
      debug_assert!(
        acquire_status == sys::Status::napi_ok,
        "Acquire threadsafe function failed in clone"
      );
    }

    Self {
      raw_tsfn: self.raw_tsfn,
      aborted: Arc::clone(&self.aborted),
      ref_count: Arc::clone(&self.ref_count),
      _phantom: PhantomData,
    }
  }
}

unsafe impl<T, R> Send for ThreadsafeFunction<T, R> {}
unsafe impl<T, R> Sync for ThreadsafeFunction<T, R> {}

impl<T: 'static, R> ThreadsafeFunction<T, R> {
  /// See [napi_create_threadsafe_function](https://nodejs.org/api/n-api.html#n_api_napi_create_threadsafe_function)
  /// for more information.
  #[allow(clippy::not_unsafe_ptr_arg_deref)]
  pub fn create<C: 'static + Send + FnMut(ThreadSafeContext<T, R>) -> Result<()>>(
    env: sys::napi_env,
    func: sys::napi_value,
    max_queue_size: usize,
    callback: C,
  ) -> Result<Self> {
    let mut async_resource_name = ptr::null_mut();
    let s = "napi_rs_threadsafe_function";
    let len = s.len();
    let s = CString::new(s)?;
    check_status!(unsafe {
      sys::napi_create_string_utf8(env, s.as_ptr(), len, &mut async_resource_name)
    })?;

    let initial_thread_count = 1usize;
    let mut raw_tsfn = ptr::null_mut();
    let ptr = Box::into_raw(Box::new(callback)) as *mut c_void;
    check_status!(unsafe {
      sys::napi_create_threadsafe_function(
        env,
        func,
        ptr::null_mut(),
        async_resource_name,
        max_queue_size,
        initial_thread_count,
        ptr,
        Some(thread_finalize_cb::<T, C, R>),
        ptr,
        Some(call_js_cb::<T, C, R>),
        &mut raw_tsfn,
      )
    })?;

    let aborted = Arc::new(AtomicBool::new(false));
    let aborted_ptr = Arc::into_raw(aborted.clone()) as *mut c_void;
    check_status!(unsafe { sys::napi_add_env_cleanup_hook(env, Some(cleanup_cb), aborted_ptr) })?;

    Ok(ThreadsafeFunction {
      raw_tsfn,
      aborted,
      ref_count: Arc::new(AtomicUsize::new(initial_thread_count)),
      _phantom: PhantomData,
    })
  }
}

impl<T: 'static, R> ThreadsafeFunction<T, R> {
  /// See [napi_call_threadsafe_function](https://nodejs.org/api/n-api.html#n_api_napi_call_threadsafe_function)
  /// for more information.
  pub fn call(
    &self,
    value: T,
    mode: ThreadsafeFunctionCallMode,
  ) -> Result<tokio::sync::oneshot::Receiver<rspack_error::Result<R>>> {
    if self.aborted.load(Ordering::Acquire) {
      return Err(napi::Error::from_status(Status::Closing));
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<rspack_error::Result<R>>();

    check_status! {
      unsafe {
        sys::napi_call_threadsafe_function(
          self.raw_tsfn,
          Box::into_raw(Box::new((value, tx))) as *mut _,
          mode.into(),
        )
      }
    };

    Ok(rx)
  }

  /// Unreferencing a threadsafe function, this might be helpful for those tsfns that are not manually dropped.
  ///
  /// See [napi_unref_threadsafe_function](https://nodejs.org/api/n-api.html#napi_unref_threadsafe_function)
  /// for more information.
  /// *Note* that in order to make sure to call this on the main thread, so a mutable reference is required.
  pub fn unref(&mut self, env: &Env) -> Result<()> {
    check_status!(unsafe { sys::napi_unref_threadsafe_function(env.raw(), self.raw_tsfn) })
  }
}

impl<T: 'static, R> Drop for ThreadsafeFunction<T, R> {
  fn drop(&mut self) {
    if !self.aborted.load(Ordering::Acquire) && self.ref_count.load(Ordering::Acquire) > 0usize {
      let release_status = unsafe {
        sys::napi_release_threadsafe_function(
          self.raw_tsfn,
          sys::ThreadsafeFunctionReleaseMode::release,
        )
      };
      assert!(
        release_status == sys::Status::napi_ok,
        "Threadsafe Function release failed"
      );
    }
  }
}

unsafe extern "C" fn cleanup_cb(cleanup_data: *mut c_void) {
  let aborted = unsafe { Arc::<AtomicBool>::from_raw(cleanup_data.cast()) };
  aborted.store(true, Ordering::SeqCst);
}

unsafe extern "C" fn thread_finalize_cb<T: 'static, C, R>(
  _raw_env: sys::napi_env,
  // context
  finalize_data: *mut c_void,
  // data
  _finalize_hint: *mut c_void,
) where
  C: 'static + Send + FnMut(ThreadSafeContext<T, R>) -> Result<()>,
{
  // cleanup
  drop(unsafe { Box::<C>::from_raw(finalize_data.cast()) });
}

unsafe extern "C" fn call_js_cb<T: 'static, C, R>(
  raw_env: sys::napi_env,
  js_callback: sys::napi_value,
  context: *mut c_void,
  data: *mut c_void,
) where
  C: 'static + Send + FnMut(ThreadSafeContext<T, R>) -> Result<()>,
{
  // env and/or callback can be null when shutting down
  if raw_env.is_null() || js_callback.is_null() {
    return;
  }

  let ctx: &mut C = unsafe { &mut *context.cast::<C>() };
  let val = Ok(*unsafe {
    Box::<(T, tokio::sync::oneshot::Sender<rspack_error::Result<R>>)>::from_raw(data.cast())
  });

  let mut recv = ptr::null_mut();
  unsafe { sys::napi_get_undefined(raw_env, &mut recv) };

  let ret = val.and_then(|v| {
    let (value, tx) = v;
    (ctx)(ThreadSafeContext {
      env: unsafe { Env::from_raw(raw_env) },
      value,
      callback: unsafe { JsFunction::from_raw(raw_env, js_callback) }
        .unwrap_or_else(|_| panic!("Threadsafe function callback is not a function")),
      tx,
    })
  });

  let status = match ret {
    Ok(()) => sys::Status::napi_ok,
    Err(e) => unsafe { sys::napi_fatal_exception(raw_env, JsError::from(e).into_value(raw_env)) },
  };
  if status == sys::Status::napi_ok {
    return;
  }
  if status == sys::Status::napi_pending_exception {
    let mut error_result = ptr::null_mut();
    assert_eq!(
      unsafe { sys::napi_get_and_clear_last_exception(raw_env, &mut error_result) },
      sys::Status::napi_ok
    );

    // When shutting down, napi_fatal_exception sometimes returns another exception
    let stat = unsafe { sys::napi_fatal_exception(raw_env, error_result) };
    assert!(stat == sys::Status::napi_ok || stat == sys::Status::napi_pending_exception);
  } else {
    let error_code: Status = status.into();
    let error_code_string = format!("{error_code:?}");
    let mut error_code_value = ptr::null_mut();
    assert_eq!(
      unsafe {
        sys::napi_create_string_utf8(
          raw_env,
          error_code_string.as_ptr() as *const _,
          error_code_string.len(),
          &mut error_code_value,
        )
      },
      sys::Status::napi_ok,
    );
    let error_msg = "Call JavaScript callback failed in thread safe function";
    let mut error_msg_value = ptr::null_mut();
    assert_eq!(
      unsafe {
        sys::napi_create_string_utf8(
          raw_env,
          error_msg.as_ptr() as *const _,
          error_msg.len(),
          &mut error_msg_value,
        )
      },
      sys::Status::napi_ok,
    );
    let mut error_value = ptr::null_mut();
    assert_eq!(
      unsafe {
        sys::napi_create_error(raw_env, error_code_value, error_msg_value, &mut error_value)
      },
      sys::Status::napi_ok,
    );
    assert_eq!(
      unsafe { sys::napi_fatal_exception(raw_env, error_value) },
      sys::Status::napi_ok
    );
  }
}

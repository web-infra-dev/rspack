use std::ffi::CStr;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Arc;

use napi::{check_status, sys, Env, Result};

struct DeferredData<Resolver: FnOnce(Env)> {
  resolver: Resolver,
}

struct JsCallbackInfo {
  tsfn: AtomicPtr<sys::napi_threadsafe_function__>,
  aborted: AtomicBool,
}

impl Drop for JsCallbackInfo {
  fn drop(&mut self) {
    if self.aborted.load(Ordering::Relaxed) {
      return;
    }
    let status = unsafe {
      sys::napi_release_threadsafe_function(
        self.tsfn.load(Ordering::Acquire),
        sys::ThreadsafeFunctionReleaseMode::release,
      )
    };
    debug_assert!(
      status == sys::Status::napi_ok,
      "Release ThreadsafeFunction in JsCallback failed"
    );
  }
}

pub struct JsCallback<Resolver: FnOnce(Env)> {
  callback_info: Arc<JsCallbackInfo>,
  _resolver: PhantomData<Resolver>,
}

unsafe impl<Resolver: FnOnce(Env)> Send for JsCallback<Resolver> {}

impl<Resolver: FnOnce(Env)> JsCallback<Resolver> {
  pub(crate) fn new(env: sys::napi_env) -> Result<Self> {
    let mut async_resource_name = ptr::null_mut();
    let s = unsafe { CStr::from_bytes_with_nul_unchecked(b"napi_js_callback\0") };
    check_status!(
      unsafe { sys::napi_create_string_utf8(env, s.as_ptr(), 16, &mut async_resource_name) },
      "Create async resource name in JsCallback failed"
    )?;

    let mut tsfn = ptr::null_mut();
    let callback_info = Arc::new(JsCallbackInfo {
      aborted: AtomicBool::new(false),
      tsfn: AtomicPtr::new(ptr::null_mut()),
    });
    check_status!(
      unsafe {
        sys::napi_create_threadsafe_function(
          env,
          ptr::null_mut(),
          ptr::null_mut(),
          async_resource_name,
          0,
          1,
          Arc::into_raw(callback_info.clone()) as _,
          Some(napi_js_finalize_cb),
          ptr::null_mut(),
          Some(napi_js_callback::<Resolver>),
          &mut tsfn,
        )
      },
      "Create threadsafe function in JsCallback failed"
    )?;
    callback_info.tsfn.store(tsfn, Ordering::Release);
    check_status!(unsafe { sys::napi_unref_threadsafe_function(env, tsfn) })?;

    let deferred = Self {
      callback_info,
      _resolver: PhantomData,
    };

    Ok(deferred)
  }

  /// The provided function will be called from the JavaScript thread
  pub fn call(&self, resolver: Resolver) {
    self.call_tsfn(resolver)
  }

  fn call_tsfn(&self, result: Resolver) {
    let data = DeferredData { resolver: result };

    // Call back into the JS thread via a threadsafe function. This results in napi_js_callback being called.
    let status = unsafe {
      sys::napi_call_threadsafe_function(
        self.callback_info.tsfn.load(Ordering::Acquire),
        Box::into_raw(Box::from(data)).cast(),
        sys::ThreadsafeFunctionCallMode::blocking,
      )
    };
    debug_assert!(
      status == sys::Status::napi_ok,
      "Call threadsafe function in JsCallback failed"
    );
  }
}

impl<Resolver: FnOnce(Env)> Clone for JsCallback<Resolver> {
  fn clone(&self) -> Self {
    if self.callback_info.aborted.load(Ordering::Relaxed) {
      panic!("JsCallback was aborted, can not clone it");
    }
    Self {
      callback_info: self.callback_info.clone(),
      _resolver: PhantomData,
    }
  }
}

extern "C" fn napi_js_finalize_cb(
  _env: sys::napi_env,
  finalize_data: *mut c_void,
  _finalize_hint: *mut c_void,
) {
  let callback_info = unsafe { Arc::<JsCallbackInfo>::from_raw(finalize_data.cast()) };
  callback_info.aborted.store(true, Ordering::Relaxed);
}

extern "C" fn napi_js_callback<Resolver: FnOnce(Env)>(
  env: sys::napi_env,
  _js_callback: sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  if env.is_null() {
    return;
  }
  let deferred_data = unsafe { Box::<DeferredData<Resolver>>::from_raw(data.cast()) };
  (deferred_data.resolver)(unsafe { Env::from_raw(env) });
}

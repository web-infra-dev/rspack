use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Arc;
use std::{ffi::CStr, sync::RwLock};

use napi::{check_status, sys, Env, Result};

struct DeferredData<Resolver: FnOnce(Env)> {
  resolver: Resolver,
}

pub struct JsCallback<Resolver: FnOnce(Env)> {
  tsfn: sys::napi_threadsafe_function,
  aborted: Arc<RwLock<bool>>,
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
    let aborted: Arc<RwLock<bool>> = Arc::new(Default::default());
    check_status!(
      unsafe {
        sys::napi_create_threadsafe_function(
          env,
          ptr::null_mut(),
          ptr::null_mut(),
          async_resource_name,
          0,
          1,
          Arc::into_raw(aborted.clone()) as _,
          Some(napi_js_finalize),
          ptr::null_mut(),
          Some(napi_js_callback::<Resolver>),
          &mut tsfn,
        )
      },
      "Create threadsafe function in JsCallback failed"
    )?;

    check_status!(unsafe { sys::napi_unref_threadsafe_function(env, tsfn) })?;

    let deferred = Self {
      tsfn,
      aborted,
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
        self.tsfn,
        Box::into_raw(Box::from(data)).cast(),
        sys::ThreadsafeFunctionCallMode::blocking,
      )
    };
    debug_assert!(
      status == sys::Status::napi_ok,
      "Call threadsafe function in JsCallback failed"
    );
  }

  fn with_aborted_read<T>(&self, f: impl FnOnce(bool) -> T) -> T {
    let aborted = self.aborted.read().expect("failed to lock aborted");
    f(*aborted)
  }
}

impl<Resolver: FnOnce(Env)> Clone for JsCallback<Resolver> {
  fn clone(&self) -> Self {
    self.with_aborted_read(|aborted| {
      if aborted {
        panic!("JsCallback was aborted, can not clone it");
      }
      Self {
        tsfn: self.tsfn,
        aborted: self.aborted.clone(),
        _resolver: self._resolver,
      }
    })
  }
}

impl<Resolver: FnOnce(Env)> Drop for JsCallback<Resolver> {
  fn drop(&mut self) {
    self.with_aborted_read(|aborted| {
      if !aborted {
        let status = unsafe {
          sys::napi_release_threadsafe_function(
            self.tsfn,
            sys::ThreadsafeFunctionReleaseMode::release,
          )
        };
        debug_assert!(
          status == sys::Status::napi_ok,
          "Release threadsafe function in JsCallback failed"
        );
      }
    })
  }
}

extern "C" fn napi_js_finalize(
  _env: sys::napi_env,
  finalize_data: *mut c_void,
  _finalize_hint: *mut c_void,
) {
  let aborted = unsafe { Arc::<RwLock<bool>>::from_raw(finalize_data.cast()) };
  let mut aborted = aborted.write().expect("failed to lock aborted");
  if !*aborted {
    *aborted = true;
  }
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

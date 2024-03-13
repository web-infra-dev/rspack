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
  ref_count: Arc<RwLock<u32>>,
  _resolver: PhantomData<Resolver>,
}

trait WithLock<T> {
  #[allow(dead_code)]
  fn with_read<R>(&self, f: impl FnOnce(&T) -> R) -> R;
  fn with_write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

impl<T> WithLock<T> for RwLock<T> {
  fn with_read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
    let lock = self.read().expect("failed to read lock");
    f(&*lock)
  }

  fn with_write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
    let mut lock = self.write().expect("failed to write lock");
    f(&mut lock)
  }
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
    let ref_count: Arc<RwLock<u32>> = Arc::new(RwLock::new(1));
    check_status!(
      unsafe {
        sys::napi_create_threadsafe_function(
          env,
          ptr::null_mut(),
          ptr::null_mut(),
          async_resource_name,
          0,
          1,
          Arc::into_raw(ref_count.clone()) as _,
          Some(napi_js_finalize_cb),
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
      ref_count,
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
}

impl<Resolver: FnOnce(Env)> Clone for JsCallback<Resolver> {
  fn clone(&self) -> Self {
    self.ref_count.with_write(|count| {
      if *count == 0 {
        panic!("JsCallback was destroyed and not able to clone");
      }
      *count += 1;
      Self {
        tsfn: self.tsfn,
        ref_count: self.ref_count.clone(),
        _resolver: self._resolver,
      }
    })
  }
}

impl<Resolver: FnOnce(Env)> Drop for JsCallback<Resolver> {
  fn drop(&mut self) {
    self.ref_count.with_write(|count| {
      if *count > 0 {
        if *count == 1 {
          let status = unsafe {
            sys::napi_release_threadsafe_function(
              self.tsfn,
              sys::ThreadsafeFunctionReleaseMode::release,
            )
          };
          debug_assert!(
            status == sys::Status::napi_ok,
            "Release ThreadsafeFunction in JsCallback failed"
          );
        }
        *count -= 1;
      }
    })
  }
}

extern "C" fn napi_js_finalize_cb(
  _env: sys::napi_env,
  finalize_data: *mut c_void,
  _finalize_hint: *mut c_void,
) {
  unsafe { Arc::<RwLock<u32>>::from_raw(finalize_data.cast()) };
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

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::{
  ffi::c_void,
  ptr,
  sync::atomic::{AtomicPtr, Ordering},
  thread::{self, ThreadId},
};

use napi::{
  bindgen_prelude::{check_status, FromNapiValue, ToNapiValue},
  sys::{self, napi_call_threadsafe_function, napi_env, napi_ref, napi_threadsafe_function__},
  Env, Result,
};

use crate::{CLEANUP_ENV_HOOK, GLOBAL_CLEANUP_FLAG};

static DELETE_REF_TS_FN: AtomicPtr<napi_threadsafe_function__> = AtomicPtr::new(ptr::null_mut());

extern "C" fn napi_js_callback(
  env: sys::napi_env,
  _js_callback: sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  // env can be null when shutting down
  if env.is_null() {
    return;
  }
  unsafe { sys::napi_delete_reference(env, data as napi_ref) };
}

pub struct ThreadsafeOneShotRef {
  env: napi_env,
  napi_ref: sys::napi_ref,
  thread_id: ThreadId,
}

impl ThreadsafeOneShotRef {
  pub fn new<T: ToNapiValue>(env: napi_env, val: T) -> Result<Self> {
    let napi_value = unsafe { ToNapiValue::to_napi_value(env, val)? };

    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, napi_value, 1, &mut napi_ref) })?;

    Self::from_napi_ref(env, napi_ref)
  }

  pub fn from_napi_ref(env: napi_env, r: sys::napi_ref) -> Result<Self> {
    let env_wrapper = Env::from(env);

    CLEANUP_ENV_HOOK.with(|ref_cell| {
      if ref_cell.borrow().is_none() {
        let result = env_wrapper.add_env_cleanup_hook((), move |_| {
          CLEANUP_ENV_HOOK.with_borrow_mut(|cleanup_env_hook| *cleanup_env_hook = None);
          GLOBAL_CLEANUP_FLAG.set(true);
        });
        if let Ok(cleanup_env_hook) = result {
          *ref_cell.borrow_mut() = Some(cleanup_env_hook);
        }
      }
    });

    if DELETE_REF_TS_FN.load(Ordering::Relaxed).is_null() {
      let mut async_resource_name = ptr::null_mut();
      check_status!(
        unsafe {
          sys::napi_create_string_utf8(
            env,
            c"delete_reference_ts_fn".as_ptr(),
            16,
            &mut async_resource_name,
          )
        },
        "Failed to create async resource name"
      )?;

      let mut ts_fn = ptr::null_mut();
      check_status!(
        unsafe {
          sys::napi_create_threadsafe_function(
            env,
            ptr::null_mut(),
            ptr::null_mut(),
            async_resource_name,
            0,
            1,
            ptr::null_mut(),
            None,
            ptr::null_mut(),
            Some(napi_js_callback),
            &mut ts_fn,
          )
        },
        "Failed to create threadsafe function"
      )?;
      check_status!(unsafe { sys::napi_unref_threadsafe_function(env, ts_fn) })?;
      DELETE_REF_TS_FN.store(ts_fn, Ordering::Relaxed);
    }

    Ok(Self {
      thread_id: thread::current().id(),
      env,
      napi_ref: r,
    })
  }
}

impl Drop for ThreadsafeOneShotRef {
  fn drop(&mut self) {
    if GLOBAL_CLEANUP_FLAG.get() {
      return;
    }
    if self.thread_id == thread::current().id() {
      unsafe { sys::napi_delete_reference(self.env, self.napi_ref) };
    } else {
      let ts_fn = DELETE_REF_TS_FN.load(Ordering::Relaxed);
      unsafe {
        let _ = napi_call_threadsafe_function(
          ts_fn,
          self.napi_ref.cast(),
          sys::ThreadsafeFunctionCallMode::nonblocking,
        );
      }
    }
  }
}

impl ToNapiValue for &ThreadsafeOneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl ToNapiValue for &mut ThreadsafeOneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl ToNapiValue for ThreadsafeOneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, &val) }
  }
}

impl FromNapiValue for ThreadsafeOneShotRef {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, napi_val, 1, &mut napi_ref) })?;

    Self::from_napi_ref(env, napi_ref)
  }
}

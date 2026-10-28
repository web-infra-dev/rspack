#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::{
  cell::RefCell,
  ffi::c_void,
  ptr,
  sync::{Arc, Weak},
  thread::{self, ThreadId},
};

use napi::{
  Result,
  bindgen_prelude::{FromNapiValue, ToNapiValue, check_status},
  sys::{self, napi_env},
  threadsafe_function::ThreadsafeFunctionHandle,
};

thread_local! {
  // Capture the owner on the JS thread. Off-thread Drop must never read TLS.
  static DELETE_REF_TS_FN: RefCell<Option<Arc<ReferenceOwner>>> = const { RefCell::new(None) };
}

struct ReferenceOwner {
  env: napi_env,
  thread_id: ThreadId,
  handle: Arc<ThreadsafeFunctionHandle>,
}

// SAFETY: env is only used on its originating JS thread. Other threads use the
// TSFN under the handle's lock, which also synchronizes with finalization.
unsafe impl Send for ReferenceOwner {}
unsafe impl Sync for ReferenceOwner {}

extern "C" fn finalize_delete_ref(_env: napi_env, data: *mut c_void, _hint: *mut c_void) {
  let handle = unsafe { Weak::<ThreadsafeFunctionHandle>::from_raw(data.cast()) };
  if let Some(handle) = handle.upgrade() {
    handle.with_write_aborted(|mut aborted| *aborted = true);
  }
}

extern "C" fn delete_reference(
  env: napi_env,
  _js_callback: sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  // Node owns reference cleanup once the environment has shut down.
  if !env.is_null() {
    unsafe { sys::napi_delete_reference(env, data.cast()) };
  }
}

impl ReferenceOwner {
  fn new(env: napi_env) -> Result<Self> {
    let mut name = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_string_utf8(env, c"delete_reference_ts_fn".as_ptr(), 21, &mut name)
    })?;
    let handle = ThreadsafeFunctionHandle::null();
    let finalize_data = scopeguard::guard(Arc::downgrade(&handle).into_raw(), |data| unsafe {
      drop(Weak::from_raw(data));
    });
    let mut raw = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_threadsafe_function(
        env,
        ptr::null_mut(),
        ptr::null_mut(),
        name,
        0,
        1,
        (*finalize_data).cast_mut().cast(),
        Some(finalize_delete_ref),
        ptr::null_mut(),
        Some(delete_reference),
        &mut raw,
      )
    })?;
    scopeguard::ScopeGuard::into_inner(finalize_data);
    // The handle releases the TSFN, including on initialization failure.
    handle.set_raw(raw);
    check_status!(unsafe { sys::napi_unref_threadsafe_function(env, raw) })?;
    Ok(Self {
      env,
      thread_id: thread::current().id(),
      handle,
    })
  }

  fn delete(&self, reference: sys::napi_ref) {
    if self.thread_id == thread::current().id() {
      self.handle.with_read_aborted(|aborted| {
        if !aborted {
          unsafe { sys::napi_delete_reference(self.env, reference) };
        }
      });
    } else {
      // Keep the guard through the native call. A Closing result must prevent
      // further calls even before Node runs the finalizer.
      self.handle.with_write_aborted(|mut aborted| {
        if *aborted {
          return;
        }
        let status = unsafe {
          sys::napi_call_threadsafe_function(
            self.handle.get_raw(),
            reference.cast(),
            sys::ThreadsafeFunctionCallMode::nonblocking,
          )
        };
        if status == sys::Status::napi_closing {
          *aborted = true;
        }
        debug_assert!(status == sys::Status::napi_ok || status == sys::Status::napi_closing);
      });
    }
  }
}

pub struct ThreadsafeOneShotRef {
  napi_ref: sys::napi_ref,
  owner: Arc<ReferenceOwner>,
}

// SAFETY: The reference is an opaque handle. Its owner routes deletion to the
// originating JS thread and prevents calls after TSFN finalization.
unsafe impl Send for ThreadsafeOneShotRef {}
// SAFETY: Shared access does not touch JS. ToNapiValue still requires the
// originating JS thread and environment, as with other Node-API values.
unsafe impl Sync for ThreadsafeOneShotRef {}

impl ThreadsafeOneShotRef {
  pub fn new<T: ToNapiValue>(env: napi_env, val: T) -> Result<Self> {
    let napi_value = unsafe { ToNapiValue::to_napi_value(env, val)? };
    unsafe { Self::from_napi_value(env, napi_value) }
  }

  pub fn from_napi_ref(env: napi_env, r: sys::napi_ref) -> Result<Self> {
    let reference_on_error = scopeguard::guard(r, |r| unsafe {
      sys::napi_delete_reference(env, r);
    });
    let owner = DELETE_REF_TS_FN.with_borrow_mut(|slot| -> Result<_> {
      if let Some(owner) = slot
        && owner.env == env
        && !owner.handle.with_read_aborted(|aborted| aborted)
      {
        return Ok(Arc::clone(owner));
      }
      let owner = Arc::new(ReferenceOwner::new(env)?);
      *slot = Some(Arc::clone(&owner));
      Ok(owner)
    })?;
    Ok(Self {
      napi_ref: scopeguard::ScopeGuard::into_inner(reference_on_error),
      owner,
    })
  }
}

impl Drop for ThreadsafeOneShotRef {
  fn drop(&mut self) {
    self.owner.delete(self.napi_ref);
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

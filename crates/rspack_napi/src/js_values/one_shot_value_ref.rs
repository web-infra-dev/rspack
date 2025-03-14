#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::{
  cell::{Cell, RefCell},
  ptr,
};

use napi::{
  bindgen_prelude::{check_status, ToNapiValue},
  sys::{self, napi_env},
  CleanupEnvHook, Env, Result,
};

thread_local! {
  static CLEANUP_ENV_HOOK: RefCell<Option<CleanupEnvHook<()>>> = Default::default();

  // cleanup references to be executed when the JS thread exits normally
  static GLOBAL_CLEANUP_FLAG: Cell<bool> = const { Cell::new(false) };
}

// A RAII (Resource Acquisition Is Initialization) style wrapper around `Ref` that ensures the
// reference is unreferenced when it goes out of scope. This struct maintains a single reference
// count and automatically cleans up when it is dropped.
pub struct OneShotRef {
  env: napi_env,
  napi_ref: sys::napi_ref,
}

impl OneShotRef {
  pub fn new<T: ToNapiValue + 'static>(env: napi_env, val: T) -> Result<Self> {
    let napi_value = unsafe { ToNapiValue::to_napi_value(env, val)? };

    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, napi_value, 1, &mut napi_ref) })?;

    Self::from_napi_ref(env, napi_ref)
  }

  pub fn from_napi_ref(env: napi_env, r: sys::napi_ref) -> Result<Self> {
    CLEANUP_ENV_HOOK.with(|ref_cell| {
      if ref_cell.borrow().is_none() {
        let env_wrapper = Env::from(env);
        let result = env_wrapper.add_env_cleanup_hook((), move |_| {
          CLEANUP_ENV_HOOK.with_borrow_mut(|cleanup_env_hook| *cleanup_env_hook = None);
          GLOBAL_CLEANUP_FLAG.set(true);
        });
        if let Ok(cleanup_env_hook) = result {
          *ref_cell.borrow_mut() = Some(cleanup_env_hook);
        }
      }
    });

    Ok(Self { env, napi_ref: r })
  }
}

impl Drop for OneShotRef {
  fn drop(&mut self) {
    if !GLOBAL_CLEANUP_FLAG.get() {
      unsafe { sys::napi_delete_reference(self.env, self.napi_ref) };
    }
  }
}

impl ToNapiValue for &OneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl ToNapiValue for &mut OneShotRef {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

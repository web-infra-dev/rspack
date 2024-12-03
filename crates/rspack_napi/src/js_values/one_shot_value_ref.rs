#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::ptr;

use napi::bindgen_prelude::{
  check_status, FromNapiMutRef, FromNapiRef, FromNapiValue, ToNapiValue,
};
use napi::sys::{self, napi_env};
use napi::{CleanupEnvHook, Env, Result};

thread_local! {
  static CLEANUP_ENV_HOOK: RefCell<Option<CleanupEnvHook<()>>> = Default::default();

  // cleanup references to be executed when the JS thread exits normally
  static GLOBAL_CLEANUP_FLAG: Cell<bool> = const { Cell::new(false) };
}

// A RAII (Resource Acquisition Is Initialization) style wrapper around `Ref` that ensures the
// reference is unreferenced when it goes out of scope. This struct maintains a single reference
// count and automatically cleans up when it is dropped.
pub struct OneShotRef<T: 'static> {
  env: napi_env,
  napi_ref: sys::napi_ref,
  ty: PhantomData<T>,
}

impl<T: ToNapiValue + 'static> OneShotRef<T> {
  pub fn new(env: napi_env, val: T) -> Result<Self> {
    let napi_value = unsafe { ToNapiValue::to_napi_value(env, val)? };

    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, napi_value, 1, &mut napi_ref) })?;

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

    Ok(Self {
      env,
      napi_ref,
      ty: PhantomData,
    })
  }
}

impl<T: FromNapiValue + ToNapiValue + 'static> OneShotRef<T> {
  pub fn from_napi_value(&self) -> Result<T> {
    let r = unsafe {
      let mut result = ptr::null_mut();
      check_status!(
        sys::napi_get_reference_value(self.env, self.napi_ref, &mut result),
        "Failed to get reference value"
      )?;
      T::from_napi_value(self.env, result)?
    };
    Ok(r)
  }
}

impl<T: FromNapiRef + ToNapiValue + 'static> OneShotRef<T> {
  pub fn from_napi_ref(&self) -> Result<&T> {
    let r = unsafe {
      let mut result = ptr::null_mut();
      check_status!(
        sys::napi_get_reference_value(self.env, self.napi_ref, &mut result),
        "Failed to get reference value"
      )?;
      T::from_napi_ref(self.env, result)?
    };
    Ok(r)
  }
}

impl<T: FromNapiMutRef + ToNapiValue + 'static> OneShotRef<T> {
  pub fn from_napi_mut_ref(&self) -> Result<&mut T> {
    let r = unsafe {
      let mut result = ptr::null_mut();
      check_status!(
        sys::napi_get_reference_value(self.env, self.napi_ref, &mut result),
        "Failed to get reference value"
      )?;
      T::from_napi_mut_ref(self.env, result)?
    };
    Ok(r)
  }
}

impl<T> Drop for OneShotRef<T> {
  fn drop(&mut self) {
    if !GLOBAL_CLEANUP_FLAG.get() {
      unsafe { sys::napi_delete_reference(self.env, self.napi_ref) };
    }
  }
}

impl<T: ToNapiValue> ToNapiValue for &OneShotRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl<T: ToNapiValue> ToNapiValue for &mut OneShotRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

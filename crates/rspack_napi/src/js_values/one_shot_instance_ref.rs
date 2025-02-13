#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;

use napi::bindgen_prelude::{check_status, ClassInstance, JavaScriptClassExt, ToNapiValue};
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
pub struct OneShotInstanceRef<T: 'static> {
  env: napi_env,
  napi_ref: sys::napi_ref,
  inner: *mut T,
  ty: PhantomData<T>,
}

impl<T: JavaScriptClassExt + 'static> OneShotInstanceRef<T> {
  pub fn new(env: napi_env, val: T) -> Result<Self> {
    let env_wrapper = Env::from_raw(env);
    let instance = val.into_instance(&env_wrapper)?;

    Self::try_from_instrance(env, instance)
  }

  pub fn try_from_instrance(env: napi_env, mut instance: ClassInstance<'_, T>) -> Result<Self> {
    let env_wrapper = Env::from_raw(env);

    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, instance.value, 1, &mut napi_ref) })?;

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

    Ok(Self {
      env,
      napi_ref,
      inner: &mut *instance,
      ty: PhantomData,
    })
  }
}

impl<T> Drop for OneShotInstanceRef<T> {
  fn drop(&mut self) {
    if !GLOBAL_CLEANUP_FLAG.get() {
      unsafe { sys::napi_delete_reference(self.env, self.napi_ref) };
    }
  }
}

impl<T: ToNapiValue> ToNapiValue for &OneShotInstanceRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl<T: ToNapiValue> ToNapiValue for &mut OneShotInstanceRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let mut result = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_get_reference_value(env, val.napi_ref, &mut result) },
      "Failed to get reference value"
    )?;
    Ok(result)
  }
}

impl<T> Deref for OneShotInstanceRef<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner }
  }
}

impl<T> DerefMut for OneShotInstanceRef<T> {
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self.inner }
  }
}

impl<T> AsRef<T> for OneShotInstanceRef<T> {
  fn as_ref(&self) -> &T {
    unsafe { &*self.inner }
  }
}

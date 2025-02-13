#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ops::{Deref, DerefMut};
use std::ptr;

use napi::bindgen_prelude::{check_status, JavaScriptClassExt, ToNapiValue};
use napi::sys::{self, napi_env};
use napi::{Env, Result};

use crate::OneShotRef;

// A RAII (Resource Acquisition Is Initialization) style wrapper around `Ref` that ensures the
// reference is unreferenced when it goes out of scope. This struct maintains a single reference
// count and automatically cleans up when it is dropped.
pub struct OneShotInstanceRef<T: 'static> {
  one_shot_ref: OneShotRef,
  inner: *mut T,
}

impl<T: JavaScriptClassExt + 'static> OneShotInstanceRef<T> {
  pub fn new(env: napi_env, val: T) -> Result<Self> {
    let env_wrapper = Env::from_raw(env);
    let mut instance = val.into_instance(&env_wrapper)?;

    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, instance.value, 1, &mut napi_ref) })?;

    Ok(Self {
      one_shot_ref: OneShotRef::from_napi_ref(env, napi_ref)?,
      inner: &mut *instance,
    })
  }
}

impl<T: ToNapiValue> ToNapiValue for &OneShotInstanceRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, &val.one_shot_ref) }
  }
}

impl<T: ToNapiValue> ToNapiValue for &mut OneShotInstanceRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, &val.one_shot_ref) }
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

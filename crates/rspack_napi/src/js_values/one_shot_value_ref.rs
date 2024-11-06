#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

use napi::bindgen_prelude::{
  check_status, FromNapiMutRef, FromNapiRef, FromNapiValue, ToNapiValue,
};
use napi::sys::{self, napi_env};
use napi::{Env, Result};

// A RAII (Resource Acquisition Is Initialization) style wrapper around `Ref` that ensures the
// reference is unreferenced when it goes out of scope. This struct maintains a single reference
// count and automatically cleans up when it is dropped.
pub struct OneShotRef<T: 'static> {
  env: napi_env,
  napi_ref: sys::napi_ref,
  cleanup_flag: Rc<RefCell<bool>>,
  ty: PhantomData<T>,
}

impl<T: ToNapiValue + 'static> OneShotRef<T> {
  pub fn new(env: napi_env, val: T) -> Result<Self> {
    let napi_value = unsafe { ToNapiValue::to_napi_value(env, val)? };

    let mut napi_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env, napi_value, 1, &mut napi_ref) })?;

    // cleanup references to be executed when the JS thread exits normally
    let cleanup_flag = Rc::new(RefCell::new(false));
    let mut env_wrapper = unsafe { Env::from_raw(env) };
    let _ = env_wrapper.add_env_cleanup_hook(cleanup_flag.clone(), move |cleanup_flag| {
      if !*cleanup_flag.borrow() {
        *cleanup_flag.borrow_mut() = true;
        unsafe { sys::napi_delete_reference(env, napi_ref) };
      }
    });

    Ok(Self {
      env,
      napi_ref,
      cleanup_flag,
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
    if !*self.cleanup_flag.borrow() {
      *self.cleanup_flag.borrow_mut() = true;
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

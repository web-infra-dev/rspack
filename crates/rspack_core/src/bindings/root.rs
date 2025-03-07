use std::{
  ffi::c_void,
  ops::{Deref, DerefMut},
};

use derive_more::Debug;
use napi::{
  bindgen_prelude::{JavaScriptClassExt, ObjectFinalize, Reference, ToNapiValue, WeakReference},
  sys::{napi_env, napi_value},
};
use napi_derive::napi;
use rspack_error::{miette::IntoDiagnostic, Result};

use crate::Compilation;

#[napi]
struct CompilationTemplate(Compilation);

#[derive(Debug)]
pub struct Root<T: 'static> {
  raw: *mut T,
  #[debug(skip)]
  reference: Reference<()>,
}

unsafe impl<T: Send> Send for Root<T> {}
unsafe impl<T: Sync> Sync for Root<T> {}

impl<T> Deref for Root<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { Box::leak(Box::from_raw(self.raw)) }
  }
}

impl<T> DerefMut for Root<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { Box::leak(Box::from_raw(self.raw)) }
  }
}

impl Root<Compilation> {
  pub fn new(val: Compilation) -> Self {
    // 只能在 js 线程调用

    let env = todo!();
    let mut instance = CompilationTemplate(val).into_instance(env).unwrap(); // TODO: use napi_throw_error
    let reference = unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, env.raw()).unwrap()
      // TODO: use napi_throw_error
    };

    Self {
      raw: &mut instance.0 as *mut _,
      reference,
    }
  }

  pub fn downgrade(&self) -> Weak<Compilation> {
    Weak {
      raw: self.raw,
      reference: self.reference.downgrade(),
    }
  }
}

impl<T> Drop for Root<T> {
  fn drop(&mut self) {
    // 不在 js 线程时，需要在 js 线程中 drop
  }
}

#[derive(Debug)]
pub struct Weak<T> {
  raw: *mut T,
  #[debug(skip)]
  reference: WeakReference<()>,
}

// Weak 不可以解引用，不安全

unsafe impl<T: Send> Send for Weak<T> {}
unsafe impl<T: Sync> Sync for Weak<T> {}

impl<T> ToNapiValue for Weak<T> {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    ToNapiValue::to_napi_value(env, val.reference)
  }
}

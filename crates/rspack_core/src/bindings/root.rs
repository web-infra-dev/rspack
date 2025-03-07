use std::{
  ffi::c_void,
  ops::{Deref, DerefMut},
};

use derive_more::Debug;
use napi::bindgen_prelude::{JavaScriptClassExt, ObjectFinalize, Reference};
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
}

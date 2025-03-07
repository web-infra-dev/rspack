use std::ffi::c_void;

use napi::{
  bindgen_prelude::{JavaScriptClassExt, Reference},
  Env,
};
use rspack_core::bindings::Root;

use crate::Compilation;

pub struct NapiAllocator(Env);

impl NapiAllocator {
  pub fn new(env: Env) -> Self {
    Self(env)
  }
}

impl rspack_core::bindings::Allocator for NapiAllocator {
  fn allocate_compilation(
    &self,
    val: rspack_core::Compilation,
  ) -> rspack_core::bindings::Root<rspack_core::Compilation> {
    let mut instance = Compilation(val).into_instance(&self.0).unwrap(); // TODO: use napi_throw_error
    let reference = unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, self.0.raw())
        .unwrap()
      // TODO: use napi_throw_error
    };
    Root::from_value_ptr(&mut instance.0 as *mut _, reference)
  }
}

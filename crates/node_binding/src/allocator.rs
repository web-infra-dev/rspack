use std::ffi::c_void;

use napi::{
  bindgen_prelude::{JavaScriptClassExt, Reference},
  Env,
};

use crate::JsCompilation;

pub struct NapiAllocator(Env);

impl NapiAllocator {
  pub fn new(env: Env) -> Self {
    Self(env)
  }
}

impl rspack_core::bindings::Allocator for NapiAllocator {
  fn allocate_compilation(&self, i: Box<rspack_core::Compilation>) -> napi::Result<Reference<()>> {
    let Ok(mut instance) = JsCompilation(i).into_instance(&self.0) else {
      return Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to allocate Compilation: unable to create instance",
      ));
    };
    let Ok(reference) = (unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, self.0.raw())
    }) else {
      return Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to allocate Compilation: unable to create reference",
      ));
    };
    Ok(reference)
  }
}

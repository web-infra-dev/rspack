use std::{
  ffi::{c_void, CString},
  ptr,
};

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
    let Ok(mut instance) = Compilation(val).into_instance(&self.0) else {
      let msg = CString::new("Failed to allocate Compilation: unable to create instance").unwrap();
      unsafe { napi::sys::napi_throw_error(self.0.raw(), ptr::null_mut(), msg.as_ptr()) };
      unreachable!()
    };
    let Ok(reference) = (unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, self.0.raw())
    }) else {
      let msg = CString::new("Failed to allocate Compilation: unable to create reference").unwrap();
      unsafe { napi::sys::napi_throw_error(self.0.raw(), ptr::null_mut(), msg.as_ptr()) };
      unreachable!()
    };
    Root::from_value_ptr(&mut instance.0 as *mut _, reference)
  }
}

mod entry_dependency;

use std::{any::TypeId, ffi::c_void, ptr};

pub use entry_dependency::*;
use napi::{
  bindgen_prelude::{FromNapiValue, Object},
  sys, Error, NapiRaw, Status,
};
use rspack_core::DependencyId;
use rspack_napi::napi::check_status;

use crate::Dependency;

// DependencyObject is used to uniformly handle Dependency and EntryDependency.
// The reason for not using Either<&Dependency, &EntryDependency> is that
// the performance of Either<&Dependency, &EntryDependency> is suboptimal.
//
// The current implementation is not perfect and has potential UB (undefined behavior) risks.
// If the passed object is neither Dependency nor EntryDependency, reading type_id may cause UB.
// The current implementation on the N-API side is similar and also has this potential risk.
//
// I have opened an issue to find a better solution: https://github.com/napi-rs/napi-rs/issues/2484
pub struct DependencyObject(Option<DependencyId>);

impl DependencyObject {
  pub fn dependency_id(&self) -> Option<DependencyId> {
    self.0
  }
}

impl FromNapiValue for DependencyObject {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let js_object = Object::from_napi_value(env, napi_val)?;

    let mut unknown_tagged_object: *mut c_void = ptr::null_mut();
    check_status!(sys::napi_unwrap(
      env,
      js_object.raw(),
      &mut unknown_tagged_object,
    ))?;

    let type_id = unknown_tagged_object as *const TypeId;
    if *type_id == TypeId::of::<Dependency>() {
      let tagged_object = &*(unknown_tagged_object as *mut Dependency);
      Ok(DependencyObject(Some(tagged_object.dependency_id)))
    } else if *type_id == TypeId::of::<EntryDependency>() {
      let tagged_object = &*(unknown_tagged_object as *mut EntryDependency);
      Ok(DependencyObject(tagged_object.dependency_id))
    } else {
      Err(Error::new(
        Status::InvalidArg,
        "Invalid argument: expected an instance of Dependency".to_owned(),
      ))
    }
  }
}

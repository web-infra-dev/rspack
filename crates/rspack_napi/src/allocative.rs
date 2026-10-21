//! Allocative-only class identity and weak roots. All N-API operations happen
//! on the owning JavaScript thread; registration never dereferences a target.
use std::{any::type_name, cell::RefCell, ffi::c_void, ptr};

use napi::{
  Env, Property, PropertyAttributes,
  bindgen_prelude::{CallbackInfo, FromNapiValue, JsObjectValue, Object, ObjectFinalize},
  sys,
};
use rspack_allocative::FlameGraphBuilder;

struct Root {
  env: sys::napi_env,
  reference: sys::napi_ref,
  address: usize,
  type_name: &'static str,
}
thread_local! {
  static ROOTS: RefCell<Vec<Root>> = const { RefCell::new(Vec::new()) };
  static CLEANUP: RefCell<Option<napi::CleanupEnvHook<()>>> = const { RefCell::new(None) };
}

/// # Safety
/// `value` must be a live class instance wrapping a `T` in `env`.
pub unsafe fn attach<T: 'static>(env: sys::napi_env, value: sys::napi_value) -> napi::Result<()> {
  let mut pointer: *mut c_void = ptr::null_mut();
  napi::check_status!(unsafe { sys::napi_unwrap(env, value, &mut pointer) })?;
  let env_wrapper = Env::from_raw(env);
  let mut object = unsafe { Object::from_napi_value(env, value)? };
  object.define_properties(&[
    Property::new()
      .with_utf8_name("__rspackRustPtr")?
      .with_property_attributes(PropertyAttributes::Default)
      .with_value(&env_wrapper.create_string(&format!("{pointer:p}"))?),
    Property::new()
      .with_utf8_name("__rspackRustType")?
      .with_property_attributes(PropertyAttributes::Default)
      .with_value(&env_wrapper.create_string(type_name::<T>())?),
  ])?;
  let mut reference = ptr::null_mut();
  napi::check_status!(unsafe { sys::napi_create_reference(env, value, 0, &mut reference) })?;
  ROOTS.with_borrow_mut(|roots| {
    roots.push(Root {
      env,
      reference,
      address: pointer as usize,
      type_name: type_name::<T>(),
    })
  });
  CLEANUP.with(|cleanup| -> napi::Result<()> {
    if cleanup.borrow().is_none() {
      *cleanup.borrow_mut() = Some(env_wrapper.add_env_cleanup_hook((), move |_| {
        ROOTS.with_borrow_mut(|roots| {
          roots.retain(|root| {
            if root.env == env {
              unsafe { sys::napi_delete_reference(env, root.reference) };
              false
            } else {
              true
            }
          });
        });
      })?);
    }
    Ok(())
  })
}

/// Called while converting the completed JS snapshot callback's return value,
/// after its GC and V8 snapshot. Dead wrappers are removed, never dereferenced.
pub fn capture(env: sys::napi_env) -> napi::Result<FlameGraphBuilder> {
  let mut builder = FlameGraphBuilder::with_shared_ownership();
  ROOTS.with(|roots| -> napi::Result<()> {
    let mut roots = roots.borrow_mut();
    let mut index = 0;
    while index < roots.len() {
      let root = &roots[index];
      if root.env != env {
        index += 1;
        continue;
      }
      let mut object = ptr::null_mut();
      napi::check_status!(unsafe {
        sys::napi_get_reference_value(env, root.reference, &mut object)
      })?;
      if object.is_null() {
        napi::check_status!(unsafe { sys::napi_delete_reference(env, root.reference) })?;
        roots.swap_remove(index);
      } else {
        // Other compilers may still be building: record identity only.
        builder.record_binding(root.address, root.type_name);
        index += 1;
      }
    }
    Ok(())
  })?;
  Ok(builder)
}

/// # Safety
/// Same requirements as napi-rs `new_instance`.
pub unsafe fn new_instance<T: ObjectFinalize + 'static>(
  env: sys::napi_env,
  pointer: *mut c_void,
  ctor: sys::napi_ref,
) -> napi::Result<sys::napi_value> {
  let value = unsafe { napi::bindgen_prelude::new_instance::<T>(env, pointer, ctor)? };
  unsafe { attach::<T>(env, value)? };
  Ok(value)
}

/// # Safety
/// `cb` and `env` must belong to the current N-API callback.
pub unsafe fn construct<const EMPTY: bool, T: ObjectFinalize + 'static, const N: usize>(
  cb: &CallbackInfo<N>,
  env: sys::napi_env,
  name: &str,
  value: T,
) -> napi::Result<sys::napi_value> {
  let instance = cb.construct::<EMPTY, T>(name, value)?;
  unsafe { attach::<T>(env, instance)? };
  Ok(instance)
}

/// # Safety
/// `cb` and `env` must belong to the current N-API callback.
pub unsafe fn factory<T: ObjectFinalize + 'static, const N: usize>(
  cb: &CallbackInfo<N>,
  env: sys::napi_env,
  name: &str,
  value: T,
) -> napi::Result<sys::napi_value> {
  let instance = cb.factory(name, value)?;
  unsafe { attach::<T>(env, instance)? };
  Ok(instance)
}

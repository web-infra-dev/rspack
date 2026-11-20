//! JavaScript owns the Module classes; napi_wrap owns their shared native handle.
use std::{
  any::TypeId,
  cell::RefCell,
  ffi::c_void,
  ptr::{self, NonNull},
  rc::Rc,
};

use napi::{Env, Result, bindgen_prelude::*, sys};
use rspack_collections::IdentifierMap;
use rspack_core::{CompilerId, ModuleIdentifier};
use rspack_napi::OneShotRef;
use rustc_hash::FxHashMap;

use crate::module::{ModuleHandle, ModuleObject};

/// JavaScript class selected when wrapping a native module.
#[doc(hidden)]
#[napi]
pub enum ModuleKind {
  Module = 0,
  Normal = 1,
  Concatenated = 2,
  Context = 3,
  External = 4,
}

impl From<TypeId> for ModuleKind {
  fn from(type_id: TypeId) -> Self {
    if type_id == TypeId::of::<rspack_core::NormalModule>() {
      Self::Normal
    } else if type_id == TypeId::of::<rspack_core::ConcatenatedModule>() {
      Self::Concatenated
    } else if type_id == TypeId::of::<rspack_core::ContextModule>() {
      Self::Context
    } else if type_id == TypeId::of::<rspack_core::ExternalModule>() {
      Self::External
    } else {
      Self::Module
    }
  }
}

static MODULE_TAG_ANCHOR: u8 = 0;

fn module_tag() -> sys::napi_type_tag {
  // Separately loaded bindings have independent compiler registries and may
  // use different ModuleHandle layouts. Their brands must also differ.
  sys::napi_type_tag {
    lower: std::ptr::addr_of!(MODULE_TAG_ANCHOR) as usize as u64,
    upper: 0x8f64_174a_63cb_92d0,
  }
}

struct Instance {
  reference: OneShotRef,
  handle: Rc<ModuleHandle>,
}

type Instances = FxHashMap<CompilerId, IdentifierMap<Instance>>;
thread_local! {
  static MODULE_INSTANCE_REFS: RefCell<Instances> = RefCell::default();
}

/// Owns direct access to the module for one invocation on this JS thread.
pub(crate) struct ModuleAccess(Rc<ModuleHandle>);

impl Drop for ModuleAccess {
  fn drop(&mut self) {
    self.0.revoke_direct_access();
  }
}

/// The caller must root `value` in the current handle scope for the entire borrow.
/// Only shared access escapes; handle mutation uses Cell/RefCell, and core module
/// mutation retains the existing loader-phase contract.
pub(crate) unsafe fn unwrap<'a>(
  env: sys::napi_env,
  value: sys::napi_value,
) -> Result<&'a ModuleHandle> {
  Ok(unsafe { unwrap_ptr(env, value)?.as_ref() })
}

/// The caller must root `value` on the owning JS thread during this call.
pub(crate) unsafe fn retain_handle(
  env: sys::napi_env,
  value: sys::napi_value,
) -> Result<Rc<ModuleHandle>> {
  let data = unwrap_ptr(env, value)?.as_ptr();
  // SAFETY: napi_wrap owns an Rc for the lifetime of the JS receiver.
  unsafe {
    Rc::increment_strong_count(data);
    Ok(Rc::from_raw(data))
  }
}

fn unwrap_ptr(env: sys::napi_env, value: sys::napi_value) -> Result<NonNull<ModuleHandle>> {
  let mut matches = false;
  check_status!(unsafe {
    sys::napi_check_object_type_tag(env, value, &module_tag(), &mut matches)
  })?;
  if !matches {
    return Err(Error::from_reason("Invalid Module receiver"));
  }
  let mut data = ptr::null_mut();
  check_status!(unsafe { sys::napi_unwrap(env, value, &mut data) })?;
  NonNull::new(data.cast()).ok_or_else(|| Error::from_reason("Invalid Module receiver"))
}

unsafe extern "C" fn finalize(_env: sys::napi_env, data: *mut c_void, _hint: *mut c_void) {
  // The cache and JS object have separate Rc owners. Environment teardown may
  // finalize JS before compiler cleanup, so neither owner may invalidate the other.
  drop(unsafe { Rc::from_raw(data.cast::<ModuleHandle>()) });
}

pub(crate) fn to_jsobject(
  env: &Env,
  mut value: ModuleObject,
) -> Result<(Object<'_>, Option<ModuleAccess>)> {
  let direct = value.has_ptr();
  let mut cache = true;
  let cached = MODULE_INSTANCE_REFS.with_borrow(|instances| -> Result<Option<_>> {
    let Some(instance) = instances
      .get(&value.compiler_id)
      .and_then(|modules| modules.get(&value.identifier))
    else {
      return Ok(None);
    };
    if direct && instance.handle.has_direct_access() {
      // Nested builds can have the same identifier. Keep each callback's
      // pointer and revocation independent.
      cache = false;
      return Ok(None);
    }
    let object = unsafe { ToNapiValue::to_napi_value(env.raw(), &instance.reference)? };
    let access = direct.then(|| {
      instance.handle.set_direct_access(&mut value);
      ModuleAccess(Rc::clone(&instance.handle))
    });
    Ok(Some((Object::from_raw(env.raw(), object), access)))
  })?;
  if let Some(object) = cached {
    return Ok(object);
  }
  let kind = ModuleKind::from(value.type_id);
  let compiler_id = value.compiler_id;
  let identifier = value.identifier;
  let handle = Rc::new(ModuleHandle::new(value));
  // Root the JS Compiler while calling its helper, and release every cache/core
  // borrow before entering JS. The helper captures no Compiler or Compilation.
  let compiler = crate::upgrade_compiler(env, compiler_id)?;
  let object = compiler.js_helpers.create_module(env, kind)?;
  wrap(env, object, &handle)?;
  let access = direct.then(|| ModuleAccess(Rc::clone(&handle)));
  if cache {
    let reference = OneShotRef::new(env.raw(), object)?;
    MODULE_INSTANCE_REFS.with_borrow_mut(|instances| {
      instances
        .entry(compiler_id)
        .or_default()
        .insert(identifier, Instance { reference, handle });
    });
  }
  Ok((object, access))
}

fn wrap(env: &Env, object: Object<'_>, handle: &Rc<ModuleHandle>) -> Result<()> {
  let data = Rc::into_raw(Rc::clone(handle));
  let status = unsafe {
    sys::napi_wrap(
      env.raw(),
      object.raw(),
      data.cast_mut().cast(),
      Some(finalize),
      ptr::null_mut(),
      ptr::null_mut(),
    )
  };
  if status != sys::Status::napi_ok {
    drop(unsafe { Rc::from_raw(data) });
    check_status!(status)?;
  }
  if let Err(error) =
    check_status!(unsafe { sys::napi_type_tag_object(env.raw(), object.raw(), &module_tag()) })
  {
    handle.revoke_direct_access();
    return Err(error);
  }
  Ok(())
}

pub(crate) fn cleanup_compiler(compiler_id: &CompilerId) {
  let removed = MODULE_INSTANCE_REFS.with_borrow_mut(|instances| instances.remove(compiler_id));
  if let Some(modules) = &removed {
    for instance in modules.values() {
      instance.handle.revoke_direct_access();
    }
  }
  drop(removed);
}

pub(crate) fn cleanup_modules(compiler_id: &CompilerId, identifiers: &[ModuleIdentifier]) {
  let removed = MODULE_INSTANCE_REFS.with_borrow_mut(|instances| {
    let mut removed = Vec::new();
    if let Some(modules) = instances.get_mut(compiler_id) {
      for identifier in identifiers {
        if let Some(instance) = modules.remove(identifier) {
          removed.push(instance);
        }
      }
    }
    removed
  });
  for instance in &removed {
    instance.handle.revoke_direct_access();
  }
  drop(removed);
}

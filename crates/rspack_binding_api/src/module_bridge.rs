//! JavaScript owns the Module classes; napi_wrap owns their shared native handle.
use std::{any::TypeId, cell::RefCell, ffi::c_void, ptr, rc::Rc};

use napi::{Env, Result, bindgen_prelude::*, sys};
use rspack_collections::IdentifierMap;
use rspack_core::{CompilerId, ModuleIdentifier};
use rspack_napi::OneShotRef;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  js_helpers::{JsModuleData, JsNormalModuleData},
  module::{ModuleHandle, ModuleObject},
  plugins::JsLoaderItem,
};

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
type Key = (usize, CompilerId, ModuleIdentifier);
thread_local! {
  static ENVIRONMENTS: RefCell<FxHashMap<usize, Instances>> = RefCell::default();
  static CREATING: RefCell<FxHashSet<Key>> = RefCell::default();
}

struct Creating(Key);
impl Drop for Creating {
  fn drop(&mut self) {
    CREATING.with_borrow_mut(|creating| creating.remove(&self.0));
  }
}

/// The caller must root `value` in the current handle scope for the entire borrow.
/// Only shared access escapes; handle mutation uses Cell/RefCell, and core module
/// mutation retains the existing loader-phase contract.
pub(crate) unsafe fn unwrap<'a>(
  env: sys::napi_env,
  value: sys::napi_value,
) -> Result<&'a ModuleHandle> {
  let mut matches = false;
  check_status!(unsafe {
    sys::napi_check_object_type_tag(env, value, &module_tag(), &mut matches)
  })?;
  if !matches {
    return Err(Error::from_reason("Invalid Module receiver"));
  }
  let mut data = ptr::null_mut();
  check_status!(unsafe { sys::napi_unwrap(env, value, &mut data) })?;
  if data.is_null() {
    return Err(Error::from_reason("Invalid Module receiver"));
  }
  Ok(unsafe { &*data.cast::<ModuleHandle>() })
}

pub struct ModuleRef<'env> {
  pub(crate) object: Object<'env>,
  handle: &'env ModuleHandle,
}

impl<'env> FromNapiValue for ModuleRef<'env> {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    Ok(Self {
      handle: unsafe { unwrap(env, value)? },
      object: unsafe { Object::from_napi_value(env, value)? },
    })
  }
}

impl std::ops::Deref for ModuleRef<'_> {
  type Target = ModuleHandle;
  fn deref(&self) -> &Self::Target {
    self.handle
  }
}

unsafe extern "C" fn finalize(_env: sys::napi_env, data: *mut c_void, _hint: *mut c_void) {
  // The cache and JS object have separate Rc owners. Environment teardown may
  // finalize JS before cache cleanup, so neither owner may invalidate the other.
  drop(unsafe { Rc::from_raw(data.cast::<ModuleHandle>()) });
}

// The callback must not invoke user JS while the identity cache is borrowed.
pub(crate) fn with_module_instance<R>(
  compiler_id: CompilerId,
  identifier: ModuleIdentifier,
  f: impl FnOnce(&ModuleHandle) -> Result<R>,
) -> Result<R> {
  ENVIRONMENTS.with_borrow(|environments| {
    let instance = environments.values().find_map(|instances| instances.get(&compiler_id)?.get(&identifier))
      .ok_or_else(|| Error::from_reason(format!(
        "Unable to access module with id = {identifier} now. The module has been removed on the Rust side or the Compiler has been garbage collected by JavaScript."
      )))?;
    f(&instance.handle)
  })
}

pub(crate) fn to_js(env: &Env, value: ModuleObject) -> Result<sys::napi_value> {
  let key = env.raw() as usize;
  let cached = ENVIRONMENTS.with_borrow(|environments| {
    environments
      .get(&key)
      .and_then(|instances| instances.get(&value.compiler_id))
      .and_then(|modules| modules.get(&value.identifier))
      .map(|instance| {
        instance.handle.ptr.set(value.ptr);
        unsafe { ToNapiValue::to_napi_value(env.raw(), &instance.reference) }
      })
      .transpose()
  })?;
  if let Some(object) = cached {
    return Ok(object);
  }
  let creation_key = (key, value.compiler_id, value.identifier);
  if !CREATING.with_borrow_mut(|creating| creating.insert(creation_key)) {
    return Err(Error::from_reason("Reentrant creation of the same Module"));
  }
  let _creating = Creating(creation_key);
  if !ENVIRONMENTS.with_borrow(|environments| environments.contains_key(&key)) {
    env.add_env_cleanup_hook(key, |key| {
      let instances = ENVIRONMENTS.with_borrow_mut(|environments| environments.remove(&key));
      if let Some(instances) = instances {
        for modules in instances.values() {
          for instance in modules.values() {
            instance.handle.ptr.set(None);
          }
        }
      }
    })?;
    ENVIRONMENTS.with_borrow_mut(|environments| environments.insert(key, Default::default()));
  }
  let kind = if value.type_id == TypeId::of::<rspack_core::NormalModule>() {
    1
  } else if value.type_id == TypeId::of::<rspack_core::ConcatenatedModule>() {
    2
  } else if value.type_id == TypeId::of::<rspack_core::ContextModule>() {
    3
  } else if value.type_id == TypeId::of::<rspack_core::ExternalModule>() {
    4
  } else {
    0
  };
  let handle = Rc::new(ModuleHandle::new(&value));
  // Preserve main's JS-owned snapshots. In particular these remain readable
  // after revocation, even when the plugin did not read them before revocation.
  let data = handle.with_ref(|_, module| {
    Ok(JsModuleData {
      identifier: env.create_string(module.identifier().as_str())?,
      module_type: env.create_string(module.module_type().as_str())?,
      normal: module
        .as_normal_module()
        .map(|normal| {
          Ok::<_, Error>(JsNormalModuleData {
            resource: env.create_string(normal.resource_resolved_data().resource())?,
            request: env.create_string(normal.request())?,
            user_request: env.create_string(normal.user_request())?,
            raw_request: env.create_string(normal.raw_request())?,
            resource_resolve_data: normal.resource_resolved_data().clone().into(),
            loaders: normal.loaders().iter().map(JsLoaderItem::from).collect(),
          })
        })
        .transpose()?,
      user_request: module
        .as_external_module()
        .map(|external| env.create_string(external.user_request()))
        .transpose()?,
    })
  })?;
  // Root the JS Compiler while calling its helper, and release every cache/core
  // borrow before entering JS. The helper captures no Compiler or Compilation.
  let compiler = crate::upgrade_compiler(env, value.compiler_id)?;
  let object = compiler.js_helpers.create_module(env, kind, data)?;
  let data = Rc::into_raw(Rc::clone(&handle));
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
    handle.ptr.set(None);
    return Err(error);
  }
  let reference = OneShotRef::new(env.raw(), object).inspect_err(|_| {
    // A helper may have retained this JS object. If cache insertion fails,
    // there will be no revocation entry to clear its loader-scoped pointer.
    handle.ptr.set(None);
  })?;
  ENVIRONMENTS.with_borrow_mut(|environments| {
    environments
      .get_mut(&key)
      .expect("Module environment is initialized")
      .entry(value.compiler_id)
      .or_default()
      .insert(value.identifier, Instance { reference, handle });
  });
  Ok(object.raw())
}

pub(crate) fn cleanup_compiler(compiler_id: &CompilerId) {
  let removed = ENVIRONMENTS.with_borrow_mut(|environments| {
    environments
      .values_mut()
      .filter_map(|instances| instances.remove(compiler_id))
      .collect::<Vec<_>>()
  });
  for modules in &removed {
    for instance in modules.values() {
      instance.handle.ptr.set(None);
    }
  }
  drop(removed);
}

pub(crate) fn cleanup_modules(compiler_id: &CompilerId, identifiers: &[ModuleIdentifier]) {
  let removed = ENVIRONMENTS.with_borrow_mut(|environments| {
    let mut removed = Vec::new();
    for instances in environments.values_mut() {
      if let Some(modules) = instances.get_mut(compiler_id) {
        for identifier in identifiers {
          if let Some(instance) = modules.remove(identifier) {
            removed.push(instance);
          }
        }
      }
    }
    removed
  });
  for instance in &removed {
    instance.handle.ptr.set(None);
  }
  drop(removed);
}

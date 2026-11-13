//! Allocative-only class identity and weak roots. All N-API operations happen
//! on the owning JavaScript thread; registration never dereferences a target.
use std::{
  any::type_name,
  cell::{Cell, RefCell},
  collections::HashMap,
  ffi::c_void,
  ptr,
  sync::atomic::{AtomicBool, Ordering},
};

use napi::{
  Env, Property, PropertyAttributes,
  bindgen_prelude::{FromNapiValue, Function, JsObjectValue, Object},
  sys,
};
use rspack_allocative::{FlameGraphBuilder, TargetIdentity};

static REGISTERED: AtomicBool = AtomicBool::new(false);

const SNAPSHOT_HOOK: &str = "__rspack_internal_runJsSnapshot";

struct Root {
  env: sys::napi_env,
  reference: sys::napi_ref,
  binding: Binding,
}

/// Snapshot a few identity values, without retaining the wrapper or target.
#[derive(Clone, Copy)]
struct Binding {
  address: usize,
  type_name: &'static str,
  target: Option<(TargetIdentity, Option<u32>)>,
}
thread_local! {
  static ROOTS: RefCell<Vec<Root>> = const { RefCell::new(Vec::new()) };
  static ENABLED_ENV: Cell<sys::napi_env> = const { Cell::new(ptr::null_mut()) };
}

type BindingMarkers = Option<Vec<(u32, String, String)>>;

/// Initialize on the first class attachment so wrappers created before
/// JsCompiler (including the resolver factory) are included in the snapshot.
fn enable_if_requested(env: &Env) -> napi::Result<bool> {
  let raw_env = env.raw();
  if ENABLED_ENV.with(|enabled| enabled.get() == raw_env) {
    return Ok(true);
  }
  if std::env::var_os("RSPACK_ALLOCATIVE_DIR").is_none()
    || !env.get_global()?.has_own_property(SNAPSHOT_HOOK)?
  {
    return Ok(false);
  }

  REGISTERED
    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
    .map_err(|_| {
      napi::Error::from_reason(
        "An allocative snapshot environment is already registered in this process",
      )
    })?;
  if let Err(error) = env.add_env_cleanup_hook((), move |_| {
    disable(raw_env);
    REGISTERED.store(false, Ordering::Release);
  }) {
    REGISTERED.store(false, Ordering::Release);
    return Err(error);
  }
  ENABLED_ENV.with(|enabled| enabled.set(raw_env));
  Ok(true)
}

/// Capture synchronously on the owning JS thread, after the build future has
/// dropped its mutable compiler borrow and while the caller pins the owner.
/// GC runs before collecting identities or visiting any native allocation.
pub fn capture(
  env: &Env,
  path: &std::path::Path,
  compiler_id: u32,
) -> napi::Result<FlameGraphBuilder> {
  // Direct binding users may never install the optional JS hook. Preserve a
  // native-only capture in that case, still starting at the full Rust owner.
  if !ENABLED_ENV.with(|enabled| enabled.get() == env.raw()) {
    return Ok(FlameGraphBuilder::with_shared_ownership());
  }
  let hook: Function<String, BindingMarkers> =
    env.get_global()?.get_named_property(SNAPSHOT_HOOK)?;
  let markers = hook.call(path.to_string_lossy().into_owned())?;
  let bindings = live_bindings(env.raw())?;
  let mut builder = FlameGraphBuilder::with_shared_ownership();
  let target_for = |binding: &Binding| {
    binding
      .target
      .and_then(|(target, owner)| (owner.is_none() || owner == Some(compiler_id)).then_some(target))
  };
  if let Some(markers) = markers {
    let bindings: HashMap<_, _> = bindings
      .into_iter()
      .map(|binding| (binding.address, binding))
      .collect();
    for (node_id, address, type_name) in markers {
      if let Some(address) = address
        .strip_prefix("0x")
        .and_then(|hex| usize::from_str_radix(hex, 16).ok())
        && let Some(binding) = bindings.get(&address)
        && binding.type_name == type_name
      {
        builder.record_js_binding_at(node_id, binding.type_name, binding.address);
        if let Some(target) = target_for(binding) {
          builder.record_js_binding_target(node_id, binding.type_name, target);
        }
      }
    }
  } else {
    for binding in bindings {
      builder.record_binding(binding.address, binding.type_name);
      if let Some(target) = target_for(&binding) {
        builder.record_binding_target(binding.address, binding.type_name, target);
      }
    }
  }
  Ok(builder)
}

fn disable(env: sys::napi_env) {
  ENABLED_ENV.with(|enabled| {
    if enabled.get() == env {
      enabled.set(ptr::null_mut());
    }
  });
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
}

/// # Safety
/// `value` must be a live class instance wrapping a `T` in `env`.
pub unsafe fn attach<T: 'static>(env: sys::napi_env, value: sys::napi_value) -> napi::Result<()> {
  let env_wrapper = Env::from_raw(env);
  if !enable_if_requested(&env_wrapper)? {
    return Ok(());
  }
  let mut pointer: *mut c_void = ptr::null_mut();
  napi::check_status!(unsafe { sys::napi_unwrap(env, value, &mut pointer) })?;
  let mut object = unsafe { Object::from_napi_value(env, value)? };
  object.define_properties(&[
    Property::new()
      .with_utf8_name("__rspackRustPtr")?
      .with_property_attributes(PropertyAttributes::Default)
      .with_value(&env_wrapper.create_string(format!("{pointer:p}"))?),
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
      binding: Binding {
        address: pointer as usize,
        type_name: type_name::<T>(),
        target: None,
      },
    })
  });
  Ok(())
}

/// Associate a final N-API wrapper with its target while the owner is accessible.
/// Only copies identity metadata; capture never dereferences this pointer.
/// Compiler-scoped targets must supply their owner ID to reject stale wrappers
/// whose address could be reused by a later compiler. Independently owned targets
/// (such as an Arc held by JsCache) need no compiler scope.
pub fn link_target<W: 'static, T: ?Sized>(wrapper: &W, target: *const T, compiler_id: Option<u32>) {
  ROOTS.with_borrow_mut(|roots| {
    if let Some(root) = roots.iter_mut().rev().find(|root| {
      root.binding.address == std::ptr::from_ref(wrapper) as usize
        && root.binding.type_name == type_name::<W>()
    }) {
      root.binding.target = Some((TargetIdentity::new(target), compiler_id));
    }
  });
}

/// Copy wrapper identities on the owning JS thread and prune dead weak refs.
fn live_bindings(env: sys::napi_env) -> napi::Result<Vec<Binding>> {
  ROOTS.with_borrow_mut(|roots| {
    let mut bindings = Vec::new();
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
        bindings.push(root.binding);
        index += 1;
      }
    }
    Ok(bindings)
  })
}

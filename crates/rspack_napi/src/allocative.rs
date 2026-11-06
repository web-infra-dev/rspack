//! Allocative-only class identity and weak roots. All N-API operations happen
//! on the owning JavaScript thread; registration never dereferences a target.
use std::{
  any::type_name,
  cell::{Cell, RefCell},
  collections::HashMap,
  ffi::c_void,
  ptr,
  sync::Arc,
};

use napi::{
  Env, JsValue, Property, PropertyAttributes,
  bindgen_prelude::{FromNapiValue, Function, JsObjectValue, Object, Promise, Unknown},
  sys,
};
use rspack_allocative::{FlameGraphBuilder, snapshot};

use crate::threadsafe_function::ThreadsafeFunction;

const SNAPSHOT_HOOK: &str = "__rspack_internal_runJsSnapshot";

struct Root {
  env: sys::napi_env,
  reference: sys::napi_ref,
  address: usize,
  type_name: &'static str,
}
thread_local! {
  static ROOTS: RefCell<Vec<Root>> = const { RefCell::new(Vec::new()) };
  static ENABLED_ENV: Cell<sys::napi_env> = const { Cell::new(ptr::null_mut()) };
}

type BindingMarkers = Option<Vec<(u32, String, String)>>;

struct PendingSnapshot {
  markers: Promise<BindingMarkers>,
  bindings: Vec<(usize, &'static str)>,
}

impl FromNapiValue for PendingSnapshot {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    // The hook has captured the V8 graph synchronously. Copy the live identities
    // before the JS thread can release wrappers while serializing that graph.
    let bindings = live_bindings(env)?;
    let markers = unsafe { Promise::from_napi_value(env, value)? };
    Ok(Self { markers, bindings })
  }
}

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

  // The TSFN retains this native dispatcher, not a JS function from globalThis.
  // Resolve the actual hook on the owning JS thread at each snapshot.
  let dispatcher = env.create_function_from_closure::<String, Unknown, _>(
    "dispatchAllocativeSnapshot",
    |ctx| {
      let hook: Function<String, Unknown> =
        ctx.env.get_global()?.get_named_property(SNAPSHOT_HOOK)?;
      hook.call(ctx.first_arg()?)
    },
  )?;
  // SAFETY: Both handles belong to the current N-API callback's environment.
  let dispatcher: ThreadsafeFunction<String, PendingSnapshot> =
    unsafe { FromNapiValue::from_napi_value(raw_env, dispatcher.raw())? };
  snapshot::register(Arc::new(move |path| {
    let dispatcher = dispatcher.clone();
    Box::pin(async move {
      let pending = dispatcher
        .call_with_sync(path.to_string_lossy().into_owned())
        .await
        .map_err(|error| error.to_string())?;
      let markers = match pending.markers.await {
        Ok(markers) => markers,
        Err(error) => return Err(dispatcher.resolve_error(error).await.to_string()),
      };
      let mut builder = FlameGraphBuilder::with_shared_ownership();
      if let Some(markers) = markers {
        let bindings: HashMap<_, _> = pending.bindings.into_iter().collect();
        for (node_id, address, type_name) in markers {
          if let Some(address) = address
            .strip_prefix("0x")
            .and_then(|hex| usize::from_str_radix(hex, 16).ok())
            && let Some(&rust_type) = bindings.get(&address)
            && rust_type == type_name
          {
            builder.record_js_binding(node_id, rust_type);
          }
        }
      } else {
        for (address, type_name) in pending.bindings {
          builder.record_binding(address, type_name);
        }
      }
      Ok(builder)
    })
  }))
  .map_err(napi::Error::from_reason)?;

  if let Err(error) = env.add_env_cleanup_hook((), move |_| {
    disable(raw_env);
    snapshot::unregister();
  }) {
    snapshot::unregister();
    return Err(error);
  }
  ENABLED_ENV.with(|enabled| enabled.set(raw_env));
  Ok(true)
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
  Ok(())
}

/// Copy wrapper identities on the owning JS thread and prune dead weak refs.
fn live_bindings(env: sys::napi_env) -> napi::Result<Vec<(usize, &'static str)>> {
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
        bindings.push((root.address, root.type_name));
        index += 1;
      }
    }
    Ok(bindings)
  })
}

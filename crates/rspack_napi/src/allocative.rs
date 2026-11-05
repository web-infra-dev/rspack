//! Allocative-only class identity and weak roots. All N-API operations happen
//! on the owning JavaScript thread; registration never dereferences a target.
use std::{
  any::type_name,
  cell::{Cell, RefCell},
  ffi::c_void,
  ptr,
  sync::Arc,
};

use napi::{
  Env, JsValue, Property, PropertyAttributes,
  bindgen_prelude::{FromNapiValue, Function, JsObjectValue, Object},
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

struct CapturedBindings(FlameGraphBuilder);

impl FromNapiValue for CapturedBindings {
  unsafe fn from_napi_value(env: sys::napi_env, _value: sys::napi_value) -> napi::Result<Self> {
    capture(env).map(Self)
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
  let dispatcher =
    env.create_function_from_closure::<String, (), _>("dispatchAllocativeSnapshot", |ctx| {
      let hook: Function<String, ()> = ctx.env.get_global()?.get_named_property(SNAPSHOT_HOOK)?;
      hook.call(ctx.first_arg()?)
    })?;
  // SAFETY: Both handles belong to the current N-API callback's environment.
  let dispatcher: ThreadsafeFunction<String, CapturedBindings> =
    unsafe { FromNapiValue::from_napi_value(raw_env, dispatcher.raw())? };
  snapshot::register(Arc::new(move |path| {
    let dispatcher = dispatcher.clone();
    Box::pin(async move {
      dispatcher
        .call_with_sync(path.to_string_lossy().into_owned())
        .await
        .map(|captured| captured.0)
        .map_err(|error| error.to_string())
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

/// Called while converting the completed JS snapshot callback's return value,
/// after its GC and V8 snapshot. Dead wrappers are removed, never dereferenced.
fn capture(env: sys::napi_env) -> napi::Result<FlameGraphBuilder> {
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

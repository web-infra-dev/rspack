#![forbid(unsafe_op_in_unsafe_fn)]

//! Allocative-only class identity and weak roots. All N-API operations happen
//! on the owning JavaScript thread; registration never dereferences a target.
use std::{
  any::{TypeId, type_name},
  cell::{Cell, RefCell},
  collections::{HashMap, HashSet},
  ffi::c_void,
  ptr,
  sync::atomic::{AtomicBool, Ordering},
};

use napi::{
  Env, Property, PropertyAttributes,
  bindgen_prelude::{FromNapiValue, Function, JsObjectValue, Object},
  sys,
};

use crate::{FlameGraphBuilder, Key, TargetIdentity, Visit, Visitor};

static REGISTERED: AtomicBool = AtomicBool::new(false);

const SNAPSHOT_HOOK: &str = "__rspack_internal_runJsSnapshot";

/// Opt out of automatic N-API root traversal. Implement beside types whose
/// contents require an explicit access window, such as an active compiler.
/// Identity registration is unaffected; the guarded caller visits these roots.
/// Ordinary classes use reflection and their field/type Allocative adapters.
pub trait ExplicitSnapshot {}

type RootVisitor = unsafe fn(*const c_void, &mut FlameGraphBuilder);

struct Root {
  env: sys::napi_env,
  reference: sys::napi_ref,
  binding: Binding,
  visit: Option<RootVisitor>,
}

/// Snapshot a few identity values, without retaining the wrapper or target.
#[derive(Clone, Copy)]
struct Binding {
  address: usize,
  type_name: &'static str,
  target: Option<(TargetIdentity, Option<u32>)>,
}
struct OwnedRoot {
  env: sys::napi_env,
  reference: sys::napi_ref,
  address: usize,
  type_name: &'static str,
  visit: unsafe fn(usize, &mut Visitor<'_>),
}

thread_local! {
  static OWNED_ROOTS: RefCell<Vec<OwnedRoot>> = const { RefCell::new(Vec::new()) };
  static ROOTS: RefCell<Vec<Root>> = const { RefCell::new(Vec::new()) };
  static ENABLED_ENV: Cell<sys::napi_env> = const { Cell::new(ptr::null_mut()) };
}

type BindingMarkers = Option<Vec<(u32, String, String)>>;

/// Initialize on the first class attachment so wrappers created before
/// JsCompiler (including the resolver factory) are included in the snapshot.
pub(crate) fn enable_if_requested(env: &Env) -> napi::Result<bool> {
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

/// Visit live classes using the typed callbacks registered by `attach`.
/// Resolving weak references pins wrappers in the caller's N-API handle scope.
/// Re-unwrap each object and check its identity before invoking the callback.
/// ExplicitSnapshot types are excluded without borrowing their Rust contents.
///
/// # Safety
/// Call synchronously on `env`'s JS thread in a N-API handle scope. Registered
/// classes must follow attach's visitation contract. In particular, types with
/// mutable borrows held outside JS callbacks must opt out via ExplicitSnapshot.
pub unsafe fn visit_live(env: &Env, builder: &mut FlameGraphBuilder) -> napi::Result<usize> {
  let pointers = ROOTS.with_borrow(|roots| {
    let mut pointers = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
      if root.env != env.raw() {
        continue;
      }
      let Some(visit) = root.visit else { continue };
      let mut object = ptr::null_mut();
      napi::check_status!(unsafe {
        sys::napi_get_reference_value(env.raw(), root.reference, &mut object)
      })?;
      if object.is_null() {
        continue;
      }
      let mut pointer = ptr::null_mut();
      napi::check_status!(unsafe { sys::napi_unwrap(env.raw(), object, &mut pointer) })?;
      if pointer as usize != root.binding.address {
        return Err(napi::Error::from_reason(
          "Allocative wrapper identity changed",
        ));
      }
      if seen.insert(pointer as usize) {
        pointers.push((pointer.cast_const(), visit));
      }
    }
    Ok::<_, napi::Error>(pointers)
  })?;
  // Release the registry borrow before entering user-defined adapters. All
  // roots share the compiler's builder, including its shared-allocation set.
  for (pointer, visit) in &pointers {
    // SAFETY: the local handle pins the allocation; attach selected its typed
    // callback and filtered types that require an explicit access window.
    unsafe { visit(*pointer, builder) };
  }
  Ok(pointers.len())
}

/// Count a separately allocated payload once, including aliases from wrappers
/// and the owner's finalizer chain. Does not traverse the JavaScript owner.
pub(crate) fn visit_shared_payload<T: 'static>(payload: &T, visitor: &mut Visitor<'_>) {
  if const { crate::reflection::vtable::<dyn ExplicitSnapshot>(TypeId::of::<T>()).is_some() } {
    return;
  }
  if let Some(mut allocation) = visitor.enter_shared(
    Key::new("payload"),
    // The pointer already belongs to the wrapper, or is profiling metadata.
    // Only the first visit contributes the separately allocated payload bytes.
    0,
    std::ptr::from_ref(payload).cast(),
  ) {
    payload.visit_memory(&mut allocation);
    allocation.exit();
  }
}

/// Register a payload whose allocation is released by a JS owner's finalizer.
/// Uses a weak owner reference, so profiling does not extend either lifetime.
///
/// # Safety
/// `owner` belongs to `env`. `payload` remains allocated and safe for immutable
/// visitation until that owner's N-API finalizer runs. Its visitor must skip
/// borrowed targets, including compilation data that may be replaced or closed.
pub(crate) unsafe fn track_owned_payload<T: 'static>(
  env: &Env,
  owner: sys::napi_value,
  payload: &T,
) -> napi::Result<()> {
  if !enable_if_requested(env)? {
    return Ok(());
  }
  if const { crate::reflection::vtable::<dyn ExplicitSnapshot>(TypeId::of::<T>()).is_some() } {
    return Ok(());
  }
  unsafe fn visit<T: 'static>(address: usize, visitor: &mut Visitor<'_>) {
    // SAFETY: registration's contract and a local owner handle pin the payload.
    visit_shared_payload(unsafe { &*(address as *const T) }, visitor);
  }
  let mut reference = ptr::null_mut();
  napi::check_status!(unsafe { sys::napi_create_reference(env.raw(), owner, 0, &mut reference) })?;
  OWNED_ROOTS.with_borrow_mut(|roots| {
    roots.push(OwnedRoot {
      env: env.raw(),
      reference,
      address: std::ptr::from_ref(payload) as usize,
      type_name: type_name::<T>(),
      visit: visit::<T>,
    });
  });
  Ok(())
}

/// Register a N-API External allocation without treating it as a wrapped class.
///
/// # Safety
/// `value` is an External<T> in `env`, and T permits immutable visitation on
/// this JS thread. Its visitor must not follow borrowed or expired targets.
pub(crate) unsafe fn track_external<T: 'static>(
  env: &Env,
  value: sys::napi_value,
) -> napi::Result<()> {
  use napi::bindgen_prelude::External;
  if const { crate::reflection::vtable::<dyn ExplicitSnapshot>(TypeId::of::<T>()).is_some() } {
    return Ok(());
  }
  if !enable_if_requested(env)? {
    return Ok(());
  }
  let mut address = ptr::null_mut();
  napi::check_status!(unsafe { sys::napi_get_value_external(env.raw(), value, &mut address) })?;
  if unsafe { External::<T>::inner_from_raw(address) }.is_none() {
    return Err(napi::Error::from_reason(
      "Allocative External type mismatch",
    ));
  }
  unsafe fn visit<T: 'static>(address: usize, visitor: &mut Visitor<'_>) {
    // The resolved local handle pins the External, including its typed payload.
    let Some(payload) = (unsafe { External::<T>::inner_from_raw(address as *mut c_void) }) else {
      visitor.report_opaque::<External<T>>();
      return;
    };
    if let Some(mut allocation) =
      visitor.enter_shared(Key::new("external"), 0, address as *const ())
    {
      let mut external = allocation.enter(
        Key::for_type_name::<External<T>>(),
        std::mem::size_of::<External<T>>(),
      );
      external.visit_field(Key::new("value"), payload);
      external.exit();
      allocation.exit();
    }
  }
  let mut reference = ptr::null_mut();
  napi::check_status!(unsafe { sys::napi_create_reference(env.raw(), value, 0, &mut reference) })?;
  OWNED_ROOTS.with_borrow_mut(|roots| {
    roots.push(OwnedRoot {
      env: env.raw(),
      reference,
      address: address as usize,
      type_name: type_name::<External<T>>(),
      visit: visit::<T>,
    })
  });
  Ok(())
}

/// Capture audited owner-finalized allocations after live wrapper traversal.
/// Weak owner resolution pins payloads inside the current N-API handle scope.
/// Dead owners are pruned before any pointer is dereferenced.
///
/// # Safety
/// Call synchronously on the owning JS thread inside a N-API handle scope.
pub unsafe fn visit_owned_payloads(env: &Env, builder: &mut FlameGraphBuilder) -> napi::Result<()> {
  let payloads = OWNED_ROOTS.with_borrow_mut(|roots| {
    let mut payloads = Vec::new();
    let mut index = 0;
    while index < roots.len() {
      let root = &roots[index];
      if root.env != env.raw() {
        index += 1;
        continue;
      }
      let mut owner = ptr::null_mut();
      napi::check_status!(unsafe {
        sys::napi_get_reference_value(env.raw(), root.reference, &mut owner)
      })?;
      if owner.is_null() {
        napi::check_status!(unsafe { sys::napi_delete_reference(env.raw(), root.reference) })?;
        roots.swap_remove(index);
      } else {
        payloads.push((root.address, root.type_name, root.visit));
        index += 1;
      }
    }
    Ok::<_, napi::Error>(payloads)
  })?;
  let mut visitor = builder.root_visitor();
  let mut owners = visitor.enter(Key::new("NapiOwnerAllocations"), 0);
  for (address, type_name, visit) in payloads {
    let mut payload = owners.enter(Key::new(type_name), 0);
    // SAFETY: audited registration plus live local owner handle.
    unsafe { visit(address, &mut payload) };
    payload.exit();
  }
  owners.exit();
  visitor.exit();
  Ok(())
}

fn disable(env: sys::napi_env) {
  OWNED_ROOTS.with_borrow_mut(|roots| {
    roots.retain(|root| {
      if root.env == env {
        unsafe { sys::napi_delete_reference(env, root.reference) };
        false
      } else {
        true
      }
    });
  });
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
/// `value` must be a live class instance wrapping a `T` in `env`. Unless T
/// implements ExplicitSnapshot, T must permit immutable visitation on the JS
/// thread between callbacks. Its adapters must not follow borrowed/expired
/// targets. Async mutable access requires an explicit snapshot policy; N-API
/// registration alone does not guarantee exclusive access to Rust contents.
pub unsafe fn attach<T: 'static>(env: sys::napi_env, value: sys::napi_value) -> napi::Result<()> {
  unsafe fn visit<T: 'static>(pointer: *const c_void, builder: &mut FlameGraphBuilder) {
    // SAFETY: the collector pins and revalidates the registered T allocation.
    builder.visit_root(unsafe { &*pointer.cast::<T>() });
  }
  // Select the policy from T without forming a reference to its contents.
  let visit: Option<RootVisitor> =
    if const { crate::reflection::vtable::<dyn ExplicitSnapshot>(TypeId::of::<T>()).is_some() } {
      None
    } else {
      Some(visit::<T>)
    };
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
      visit,
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

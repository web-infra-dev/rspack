use std::{
  any::TypeId,
  cell::RefCell,
  ffi::c_void,
  marker::PhantomData,
  ptr,
  sync::{Arc, LazyLock, Mutex},
};

use napi::{
  Env, Result,
  bindgen_prelude::{ClassInstance, FromNapiValue, JavaScriptClassExt},
  check_status, sys,
};
use rustc_hash::FxHashMap as HashMap;

use crate::LifecycleId;

type DropEvent = (TypeId, u64);

// Only queues and TSFN handles cross threads. All references remain in ENVS.
static DISPATCHERS: LazyLock<Mutex<HashMap<usize, Arc<Dispatcher>>>> =
  LazyLock::new(Default::default);

thread_local! {
  static ENVS: RefCell<HashMap<usize, EnvReferences>> = RefCell::default();
}

struct References {
  lifetime_type: TypeId,
  values: HashMap<u64, sys::napi_ref>,
}

struct EnvReferences {
  types: HashMap<TypeId, References>,
}

#[derive(Default)]
struct DispatchState {
  // Serialized with enqueue and environment shutdown; never dereferenced in Rust.
  tsfn: Option<usize>,
  pending: Vec<DropEvent>,
  scheduled: bool,
}

#[derive(Default)]
struct Dispatcher(Mutex<DispatchState>);

impl Dispatcher {
  fn enqueue(self: &Arc<Self>, event: DropEvent) {
    let mut state = self.0.lock().expect("reference cleanup queue lock");
    let Some(tsfn) = state.tsfn else { return };
    state.pending.push(event);
    if state.scheduled {
      return;
    }
    state.scheduled = true;
    let data = Box::into_raw(Box::new(self.clone()));
    // An unlimited, nonblocking queue also works when a guard drops on the JS
    // thread. Batch notifications until this callback drains them.
    let status = unsafe {
      sys::napi_call_threadsafe_function(
        tsfn as sys::napi_threadsafe_function,
        data.cast(),
        sys::ThreadsafeFunctionCallMode::nonblocking,
      )
    };
    if status != sys::Status::napi_ok {
      // Closing environments may reject notifications; their cleanup hook
      // releases every reference. No queued allocation may escape on failure.
      unsafe { drop(Box::from_raw(data)) };
      state.pending.clear();
      state.scheduled = false;
    }
  }
}

pub(crate) fn notify_drop(lifetime_type: TypeId, id: u64) {
  for dispatcher in DISPATCHERS
    .lock()
    .expect("reference dispatcher lock")
    .values()
  {
    dispatcher.enqueue((lifetime_type, id));
  }
}

extern "C" fn cleanup_callback(
  env: sys::napi_env,
  _callback: sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  // Node also drains queued data with a null env during shutdown.
  let dispatcher = unsafe { Box::<Arc<Dispatcher>>::from_raw(data.cast()) };
  let events = {
    let mut state = dispatcher.0.lock().expect("reference cleanup queue lock");
    state.scheduled = false;
    std::mem::take(&mut state.pending)
  };
  if env.is_null() {
    return;
  }
  ENVS.with_borrow_mut(|envs| {
    if let Some(cache) = envs.get_mut(&(env as usize)) {
      for (lifetime_type, id) in events {
        for references in cache.types.values_mut() {
          if references.lifetime_type == lifetime_type
            && let Some(reference) = references.values.remove(&id)
          {
            unsafe { sys::napi_delete_reference(env, reference) };
          }
        }
      }
    }
  });
}

fn initialize(env: &Env) -> Result<()> {
  let key = env.raw() as usize;
  if ENVS.with_borrow(|envs| envs.contains_key(&key)) {
    return Ok(());
  }
  let dispatcher = Arc::new(Dispatcher::default());
  let mut name = ptr::null_mut();
  check_status!(unsafe {
    sys::napi_create_string_utf8(env.raw(), c"lifecycle_cleanup".as_ptr(), 17, &mut name)
  })?;
  let mut tsfn = ptr::null_mut();
  check_status!(unsafe {
    sys::napi_create_threadsafe_function(
      env.raw(),
      ptr::null_mut(),
      ptr::null_mut(),
      name,
      0,
      1,
      ptr::null_mut(),
      None,
      ptr::null_mut(),
      Some(cleanup_callback),
      &mut tsfn,
    )
  })?;
  let setup = (|| {
    check_status!(unsafe { sys::napi_unref_threadsafe_function(env.raw(), tsfn) })?;
    env.add_env_cleanup_hook(key, |key| {
      if let Some(dispatcher) = DISPATCHERS
        .lock()
        .expect("reference dispatcher lock")
        .remove(&key)
      {
        let mut state = dispatcher.0.lock().expect("reference cleanup queue lock");
        if let Some(tsfn) = state.tsfn.take() {
          unsafe {
            sys::napi_release_threadsafe_function(
              tsfn as sys::napi_threadsafe_function,
              sys::ThreadsafeFunctionReleaseMode::release,
            )
          };
        }
        state.pending.clear();
      }
      // Remove the cache before deleting references, including for environments
      // with no subsequent loader invocation (hook failure, cancellation, exit).
      let cache = ENVS.with_borrow_mut(|envs| envs.remove(&key));
      if let Some(cache) = cache {
        for references in cache.types.into_values() {
          for reference in references.values.into_values() {
            unsafe { sys::napi_delete_reference(key as sys::napi_env, reference) };
          }
        }
      }
    })?;
    Ok(())
  })();
  if let Err(error) = setup {
    unsafe {
      sys::napi_release_threadsafe_function(tsfn, sys::ThreadsafeFunctionReleaseMode::release)
    };
    return Err(error);
  }
  dispatcher
    .0
    .lock()
    .expect("reference cleanup queue lock")
    .tsfn = Some(tsfn as usize);
  ENVS.with_borrow_mut(|envs| {
    envs.insert(
      key,
      EnvReferences {
        types: HashMap::default(),
      },
    );
  });
  DISPATCHERS
    .lock()
    .expect("reference dispatcher lock")
    .insert(key, dispatcher);
  Ok(())
}

/// Reuses a JS class for a native lifetime on the current JS thread and env.
/// References are released asynchronously when the guard drops, or at env exit.
/// `T` identifies the native lifetime; `J` identifies its JS representation.
pub struct ThreadLocalReference<T: 'static, J: 'static>(PhantomData<fn() -> (T, J)>);

impl<T: 'static, J: JavaScriptClassExt + 'static> ThreadLocalReference<T, J> {
  /// The caller must keep the corresponding guard alive throughout this call.
  /// Returned instances follow the class's own native access-window policy.
  pub fn get_or_insert_with<'env>(
    env: &'env Env,
    id: LifecycleId<T>,
    create: impl FnOnce() -> J,
  ) -> Result<ClassInstance<'env, J>> {
    initialize(env)?;
    let key = env.raw() as usize;
    let cached = ENVS.with_borrow(|envs| {
      envs
        .get(&key)
        .and_then(|cache| cache.types.get(&TypeId::of::<(T, J)>()))
        .and_then(|references| references.values.get(&id.value).copied())
    });
    if let Some(reference) = cached {
      let mut value = ptr::null_mut();
      check_status!(unsafe { sys::napi_get_reference_value(env.raw(), reference, &mut value) })?;
      return unsafe { ClassInstance::from_napi_value(env.raw(), value) };
    }
    // Do not hold a RefCell borrow while constructing a native-backed class.
    let instance = create().into_instance(env)?;
    let mut reference = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_reference(env.raw(), instance.value, 1, &mut reference)
    })?;
    ENVS.with_borrow_mut(|envs| {
      envs
        .get_mut(&key)
        .expect("initialized reference cache")
        .types
        .entry(TypeId::of::<(T, J)>())
        .or_insert_with(|| References {
          lifetime_type: TypeId::of::<T>(),
          values: HashMap::default(),
        })
        .values
        .insert(id.value, reference);
    });
    Ok(instance)
  }
}

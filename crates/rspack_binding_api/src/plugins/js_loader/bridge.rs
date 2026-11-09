use std::sync::{Arc, LazyLock, Mutex, Weak};

use napi::{
  Env, Status,
  bindgen_prelude::Buffer,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use rspack_js_tasks::LoaderTaskMode;
use rspack_loader_runner::JsLoaderRuntime;
use rustc_hash::FxHashMap;

type ReleaseCallback = ThreadsafeFunction<u32, (), u32, Status, false, true, 0>;

// The env owns its route until cleanup. Handles retain that exact route, so an
// old handle can never release the same numeric ID in a replacement env.
static RELEASE: Mutex<Weak<MainObjectRelease>> = Mutex::new(Weak::new());

struct MainObjectRelease {
  callback: Mutex<Option<ReleaseCallback>>,
}

struct MainObjectHandleInner {
  handle: u32,
  release: Arc<MainObjectRelease>,
}

impl Drop for MainObjectHandleInner {
  fn drop(&mut self) {
    if let Some(callback) = self
      .release
      .callback
      .lock()
      .expect("main object release lock poisoned")
      .as_ref()
    {
      callback.call(self.handle, ThreadsafeFunctionCallMode::NonBlocking);
    }
  }
}

/// A cloneable handle for a JavaScript value owned by the main thread.
///
/// The last clone releases the handle through a threadsafe main-thread callback.
#[derive(Clone)]
pub struct MainObjectHandle {
  inner: Arc<MainObjectHandleInner>,
}

impl MainObjectHandle {
  pub fn new(handle: u32) -> Self {
    Self {
      inner: Arc::new(MainObjectHandleInner {
        handle,
        release: RELEASE
          .lock()
          .expect("main object release lock poisoned")
          .upgrade()
          .expect("main-thread JS value release callback should be registered"),
      }),
    }
  }

  pub fn handle(&self) -> u32 {
    self.inner.handle
  }
}

struct LoaderReferences {
  runtime: Arc<JsLoaderRuntime>,
  serialized_options: FxHashMap<String, Vec<u8>>,
  pure_compiler_options: Option<Vec<u8>>,
}

impl LoaderReferences {
  fn new(owner: u32) -> Self {
    Self {
      runtime: Arc::new(JsLoaderRuntime::new(owner)),
      serialized_options: Default::default(),
      pure_compiler_options: None,
    }
  }
}

/// V8 bytes owned by the native loader context, independent of any JS env.
pub(super) struct SerializedLoaderData(pub Vec<u8>);

static REFERENCES: LazyLock<Mutex<FxHashMap<u32, LoaderReferences>>> =
  LazyLock::new(Default::default);

#[napi(ts_args_type = "release: (value: number) => void")]
pub fn register_main_object_release(env: Env, release: ReleaseCallback) -> napi::Result<()> {
  let mut global = RELEASE.lock().expect("main object release lock poisoned");
  if let Some(current) = global.upgrade()
    && current
      .callback
      .lock()
      .expect("main object release lock poisoned")
      .is_some()
  {
    return Err(napi::Error::from_reason(
      "Main object release callback already belongs to a live JS env",
    ));
  }
  let route = Arc::new(MainObjectRelease {
    callback: Mutex::new(Some(release)),
  });
  env.add_env_cleanup_hook(route.clone(), |route| {
    // Serialize with handle Drop before releasing the TSFN. The route remains
    // inert even if Rust-owned handles outlive this JS env.
    let callback = route
      .callback
      .lock()
      .expect("main object release lock poisoned")
      .take();
    let mut global = RELEASE.lock().expect("main object release lock poisoned");
    if global.ptr_eq(&Arc::downgrade(&route)) {
      *global = Weak::new();
      REFERENCES
        .lock()
        .expect("loader references lock poisoned")
        .clear();
    }
    drop(global);
    drop(callback);
  })?;
  *global = Arc::downgrade(&route);
  Ok(())
}

#[napi]
pub fn register_loader_reference(
  owner: u32,
  ident: String,
  parallel: bool,
  serialized: Option<Buffer>,
  pure: Option<bool>,
  pure_compiler_options: Option<Buffer>,
) -> bool {
  let parallel = parallel && !cfg!(target_family = "wasm");
  let mut references = REFERENCES.lock().expect("loader references lock poisoned");
  let references = references
    .entry(owner)
    .or_insert_with(|| LoaderReferences::new(owner));
  if let Some(serialized) = serialized {
    references
      .serialized_options
      .insert(ident.clone(), serialized.to_vec());
  } else {
    references.serialized_options.remove(&ident);
  }
  if let Some(options) = pure_compiler_options {
    references.pure_compiler_options = Some(options.to_vec());
  }
  let mode = if !parallel {
    LoaderTaskMode::Main
  } else if pure.unwrap_or_default() {
    LoaderTaskMode::Pure
  } else {
    LoaderTaskMode::Parallel
  };
  references.runtime.register_loader(ident, mode);
  parallel
}

/// Read cached V8 bytes in the calling worker env, without a main-thread RPC.
#[napi]
pub fn get_serialized_loader_options(owner: u32, ident: Option<String>) -> Option<Buffer> {
  let references = REFERENCES.lock().expect("loader references lock poisoned");
  let references = references.get(&owner)?;
  let bytes = match ident {
    Some(ident) => references.serialized_options.get(&ident)?,
    None => references.pure_compiler_options.as_ref()?,
  };
  Some(Buffer::from(bytes.clone()))
}

#[napi]
pub fn release_loader_references(owner: u32) {
  REFERENCES
    .lock()
    .expect("loader references lock poisoned")
    .remove(&owner);
}

pub(super) fn loader_runtime(owner: u32) -> Arc<JsLoaderRuntime> {
  REFERENCES
    .lock()
    .expect("loader references lock poisoned")
    .entry(owner)
    .or_insert_with(|| LoaderReferences::new(owner))
    .runtime
    .clone()
}

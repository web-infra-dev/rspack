use std::sync::{Arc, LazyLock, Mutex};

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use rustc_hash::FxHashMap;

use super::runtime::JsLoaderRuntime;

struct LoaderReferences {
  runtime: Arc<JsLoaderRuntime>,
  serialized_options: FxHashMap<String, Vec<u8>>,
  compiler_options: Option<Vec<u8>>,
}

impl LoaderReferences {
  fn new() -> Self {
    Self {
      runtime: Arc::new(JsLoaderRuntime::new()),
      serialized_options: Default::default(),
      compiler_options: None,
    }
  }
}

/// V8 bytes owned by the native loader context, independent of any JS env.
pub(super) struct SerializedLoaderData(pub Vec<u8>);

static REFERENCES: LazyLock<Mutex<FxHashMap<u32, LoaderReferences>>> =
  LazyLock::new(Default::default);

#[napi]
pub fn register_loader_reference(
  owner: u32,
  ident: String,
  parallel: bool,
  serialized: Option<Buffer>,
  strict: Option<bool>,
  compiler_options: Option<Buffer>,
) -> bool {
  let parallel = parallel && !cfg!(target_family = "wasm");
  let mut references = REFERENCES.lock().expect("loader references lock poisoned");
  let references = references
    .entry(owner)
    .or_insert_with(LoaderReferences::new);
  if let Some(options) = compiler_options {
    references.compiler_options = Some(options.to_vec());
  }
  if let Some(serialized) = serialized {
    references
      .serialized_options
      .insert(ident.clone(), serialized.to_vec());
  } else {
    references.serialized_options.remove(&ident);
  }
  references
    .runtime
    .register_loader(ident, parallel, strict.unwrap_or(false));
  parallel
}

/// Read cached V8 bytes in the calling worker env, without a main-thread RPC.
#[napi]
pub fn get_serialized_loader_options(owner: u32, ident: Option<String>) -> Option<Buffer> {
  let references = REFERENCES.lock().expect("loader references lock poisoned");
  let references = references.get(&owner)?;
  let bytes = match ident {
    Some(ident) => references.serialized_options.get(&ident)?,
    None => references.compiler_options.as_ref()?,
  };
  Some(Buffer::from(bytes.clone()))
}

pub(super) fn release_loader_references(owner: u32) {
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
    .or_insert_with(LoaderReferences::new)
    .runtime
    .clone()
}

use std::sync::{LazyLock, Mutex};

use napi::bindgen_prelude::Buffer;
use rustc_hash::FxHashMap;

#[derive(Default)]
struct LoaderReferences {
  options: FxHashMap<String, Vec<u8>>,
  compiler_options: Option<Vec<u8>>,
}

static REFERENCES: LazyLock<Mutex<FxHashMap<u32, LoaderReferences>>> =
  LazyLock::new(Default::default);

#[napi]
pub fn register_strict_loader(
  owner: u32,
  ident: String,
  options: Buffer,
  compiler_options: Option<Buffer>,
) {
  let mut references = REFERENCES.lock().expect("loader references lock poisoned");
  let references = references.entry(owner).or_default();
  references.options.insert(ident, options.to_vec());
  if let Some(options) = compiler_options {
    references.compiler_options = Some(options.to_vec());
  }
}

pub(super) fn strict_loaders(owner: u32, f: impl FnOnce(Option<&FxHashMap<String, Vec<u8>>>)) {
  let references = REFERENCES.lock().expect("loader references lock poisoned");
  f(references.get(&owner).map(|references| &references.options));
}

#[napi]
pub fn get_serialized_loader_options(owner: u32, ident: Option<String>) -> Option<Buffer> {
  let references = REFERENCES.lock().expect("loader references lock poisoned");
  let references = references.get(&owner)?;
  let bytes = match ident {
    Some(ident) => references.options.get(&ident)?,
    None => references.compiler_options.as_ref()?,
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

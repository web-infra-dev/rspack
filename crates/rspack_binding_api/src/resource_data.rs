use std::{
  cell::RefCell,
  path::{Path, PathBuf},
  sync::{Arc, LazyLock, Mutex, Weak},
};

use napi::{
  Env, JsValue, Property,
  bindgen_prelude::{JavaScriptClassExt, JsObjectValue, ToNapiValue},
  sys::napi_value,
};
use napi_derive::napi;
use rspack_fs::ReadableFileSystem;
use rspack_paths::Utf8Path;
use rustc_hash::FxHashMap;

// Converting the descriptionFileData property to a JSObject may become a performance bottleneck.
// Additionally, descriptionFileData and descriptionFilePath are rarely used, so they are exposed via getter methods and only converted to JSObject when accessed.
#[napi]
pub struct ReadonlyResourceData {
  i: Weak<rspack_core::ResourceData>,
  /// Input filesystem of the compilation that resolved the resource, used for
  /// descriptions the resolver did not materialize.
  input_filesystem: Arc<dyn ReadableFileSystem>,
}

impl ReadonlyResourceData {
  pub fn with_ref<R>(
    &self,
    f: impl FnOnce(&rspack_core::ResourceData) -> napi::Result<R>,
  ) -> napi::Result<R> {
    match self.i.upgrade() {
      Some(arc) => f(arc.as_ref()),
      None => Err(napi::Error::from_reason(
        "ResourceData has been dropped by Rust.",
      )),
    }
  }
}

#[napi]
impl ReadonlyResourceData {
  #[napi(getter, ts_return_type = "any")]
  pub fn description_file_data(&self, env: &Env) -> napi::Result<Option<napi_value>> {
    self.with_ref(|resource_data| {
      resource_data
        .description()
        .and_then(|description| description_json(description, &self.input_filesystem))
        .map(|json| unsafe { ToNapiValue::to_napi_value(env.raw(), json) })
        .transpose()
    })
  }

  #[napi(getter, ts_return_type = "string")]
  pub fn description_file_path(&self, env: &Env) -> napi::Result<Option<napi_value>> {
    self.with_ref(|resource_data| {
      resource_data
        .description()
        .map(|data| unsafe {
          ToNapiValue::to_napi_value(env.raw(), data.path().to_string_lossy().as_ref())
        })
        .transpose()
    })
  }
}

pub struct ReadonlyResourceDataWrapper {
  i: Arc<rspack_core::ResourceData>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
}

impl ReadonlyResourceDataWrapper {
  /// `input_filesystem` serves descriptions the resolver did not materialize.
  pub fn new(
    i: Arc<rspack_core::ResourceData>,
    input_filesystem: Arc<dyn ReadableFileSystem>,
  ) -> Self {
    Self {
      i,
      input_filesystem,
    }
  }
}

thread_local! {
  static RESOURCE_DATA_PROPERTIES_BUFFER: RefCell<Vec<Property>> = const { RefCell::new(Vec::new()) };
}

impl ToNapiValue for ReadonlyResourceDataWrapper {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let env_wrapper = Env::from_raw(env);

    let resource_data = val.i;
    let template = ReadonlyResourceData {
      i: Arc::downgrade(&resource_data),
      input_filesystem: val.input_filesystem,
    };
    let instance = template.into_instance(&env_wrapper)?;
    let mut object = instance.as_object(&env_wrapper);

    RESOURCE_DATA_PROPERTIES_BUFFER.with(|ref_cell| {
      let mut properties = ref_cell.borrow_mut();
      properties.clear();
      properties.push(
        Property::new()
          .with_utf8_name("resource")?
          .with_value(&env_wrapper.create_string(resource_data.resource())?),
      );
      if let Some(path) = resource_data.path() {
        properties.push(
          Property::new()
            .with_utf8_name("path")?
            .with_value(&env_wrapper.create_string(path.as_str())?),
        );
      }
      if let Some(query) = resource_data.query() {
        properties.push(
          Property::new()
            .with_utf8_name("query")?
            .with_value(&env_wrapper.create_string(query)?),
        );
      }
      if let Some(fragment) = resource_data.fragment() {
        properties.push(
          Property::new()
            .with_utf8_name("fragment")?
            .with_value(&env_wrapper.create_string(fragment)?),
        );
      }
      object.define_properties(&properties)
    })?;

    Ok(object.raw())
  }
}

#[napi(object)]
pub struct JsResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub path: Option<String>,
  /// Resource query with `?` prefix
  pub query: Option<String>,
  /// Resource fragment with `#` prefix
  pub fragment: Option<String>,
  pub description_file_data: Option<serde_json::Value>,
  pub description_file_path: Option<String>,
}

/// One `package.json` that a JS consumer asked for.
struct CachedDescription {
  /// Filesystem the description was read with, so a recycled allocation cannot
  /// alias content another filesystem produced.
  fs: Weak<dyn ReadableFileSystem>,

  /// Modification time the description was read at, when the filesystem
  /// reports one.
  modified: u64,

  value: Arc<serde_json::Value>,
}

/// Number of cached descriptions to keep before dropping the entries whose
/// filesystem is gone.
const DESCRIPTION_JSON_CACHE_LIMIT: usize = 4096;

/// Descriptions JS consumers asked for, keyed by the filesystem that read them
/// and the described file. Entries are revalidated against the file's
/// modification time, so watch mode and long-lived processes pick up edits, and
/// they are dropped once the filesystem that produced them is gone.
static DESCRIPTION_JSON_CACHE: LazyLock<Mutex<FxHashMap<(usize, PathBuf), CachedDescription>>> =
  LazyLock::new(Default::default);

fn description_cache_key(fs: &Arc<dyn ReadableFileSystem>, path: &Path) -> (usize, PathBuf) {
  (Arc::as_ptr(fs) as *const () as usize, path.to_path_buf())
}

fn cached_description(
  fs: &Arc<dyn ReadableFileSystem>,
  path: &Path,
  modified: u64,
) -> Option<Arc<serde_json::Value>> {
  let cache = DESCRIPTION_JSON_CACHE
    .lock()
    .unwrap_or_else(|err| err.into_inner());
  let entry = cache.get(&description_cache_key(fs, path))?;
  if entry.modified != modified
    || !entry
      .fs
      .upgrade()
      .is_some_and(|entry_fs| Arc::ptr_eq(&entry_fs, fs))
  {
    return None;
  }
  Some(entry.value.clone())
}

fn cache_description(
  fs: &Arc<dyn ReadableFileSystem>,
  path: &Path,
  modified: u64,
  value: Arc<serde_json::Value>,
) {
  let mut cache = DESCRIPTION_JSON_CACHE
    .lock()
    .unwrap_or_else(|err| err.into_inner());
  if cache.len() >= DESCRIPTION_JSON_CACHE_LIMIT {
    cache.retain(|_, entry| entry.fs.strong_count() > 0);
  }
  cache.insert(
    description_cache_key(fs, path),
    CachedDescription {
      fs: Arc::downgrade(fs),
      modified,
      value,
    },
  );
}

/// Reads a `package.json` through the filesystem the compilation resolved it
/// with, reusing the parsed value while the file is unchanged.
fn read_description_json(
  fs: &Arc<dyn ReadableFileSystem>,
  path: &Path,
) -> Option<Arc<serde_json::Value>> {
  let path = Utf8Path::from_path(path)?;
  let modified = fs.metadata_sync(path).ok().map(|meta| meta.mtime_ms);
  if let Some(modified) = modified
    && let Some(json) = cached_description(fs, path.as_std_path(), modified)
  {
    return Some(json);
  }
  let source = fs.read_to_string_sync(path).ok()?;
  let json = Arc::new(serde_json::from_str::<serde_json::Value>(&source).ok()?);
  if let Some(modified) = modified {
    cache_description(fs, path.as_std_path(), modified, json.clone());
  }
  Some(json)
}

/// `package.json` content of a description for JS consumers: the mirror the
/// resolver materialized when it kept one, otherwise read from the described
/// file through the input filesystem of the compilation.
///
/// The read is synchronous because JS getters run on the JS thread. Filesystems
/// that can only be read by calling back into JS therefore keep the mirror
/// instead (see `JsCompiler::new`), so this never has to call into JS.
pub(crate) fn description_json(
  description: &rspack_loader_runner::DescriptionData,
  fs: &Arc<dyn ReadableFileSystem>,
) -> Option<serde_json::Value> {
  let json = description.json();
  if !json.is_null() {
    return Some(json.clone());
  }
  read_description_json(fs, description.json_path()).map(|json| json.as_ref().clone())
}

impl JsResourceData {
  /// Builds the JS-facing resource data. Descriptions the resolver did not
  /// materialize are read through the input filesystem of the compilation.
  pub(crate) fn from_resource_data(
    value: &rspack_core::ResourceData,
    fs: &Arc<dyn ReadableFileSystem>,
  ) -> Self {
    Self {
      resource: value.resource().to_owned(),
      path: value.path().map(|p| p.as_str().to_string()),
      fragment: value.fragment().map(|r| r.to_owned()),
      query: value.query().map(|r| r.to_owned()),
      description_file_data: value
        .description()
        .and_then(|description| description_json(description, fs)),
      description_file_path: value
        .description()
        .map(|data| data.path().to_string_lossy().into_owned()),
    }
  }
}

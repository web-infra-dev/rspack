use std::{
  cell::RefCell,
  sync::{Arc, Weak},
};

use napi::{
  Env, JsValue, Property,
  bindgen_prelude::{JavaScriptClassExt, JsObjectValue, ToNapiValue},
  sys::napi_value,
};
use napi_derive::napi;

// Converting the descriptionFileData property to a JSObject may become a performance bottleneck.
// Additionally, descriptionFileData and descriptionFilePath are rarely used, so they are exposed via getter methods and only converted to JSObject when accessed.
#[napi]
pub struct ReadonlyResourceData {
  i: Weak<rspack_core::ResourceData>,
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
        .and_then(description_file_json)
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

impl From<Arc<rspack_core::ResourceData>> for ReadonlyResourceDataWrapper {
  fn from(value: Arc<rspack_core::ResourceData>) -> Self {
    ReadonlyResourceDataWrapper { i: value }
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

/// Cached `(modification time, parsed package.json)` for one description file.
type CachedDescriptionJson = (Option<std::time::SystemTime>, Option<serde_json::Value>);

/// Process-wide cache of package.json descriptions that JS consumers asked for.
type DescriptionJsonCache =
  std::sync::Mutex<rustc_hash::FxHashMap<std::path::PathBuf, CachedDescriptionJson>>;

/// Reads a package.json description for JS consumers. The resolver only
/// materializes descriptions that the compilation observes, so this falls back
/// to reading the file when JS asks for one that was not collected.
pub(crate) fn read_description_file_json(path: &std::path::Path) -> Option<serde_json::Value> {
  // The same package.json backs many modules, so cache the parsed value;
  // validate it against the file's modification time so watch mode and
  // long-lived processes pick up edits.
  static CACHE: std::sync::LazyLock<DescriptionJsonCache> =
    std::sync::LazyLock::new(Default::default);
  let modified = std::fs::metadata(path)
    .and_then(|meta| meta.modified())
    .ok();
  let mut cache = CACHE.lock().unwrap_or_else(|err| err.into_inner());
  if let Some((cached_modified, value)) = cache.get(path)
    && *cached_modified == modified
  {
    return value.clone();
  }
  let value = std::fs::read_to_string(path)
    .ok()
    .and_then(|source| serde_json::from_str(&source).ok());
  cache.insert(path.to_path_buf(), (modified, value.clone()));
  value
}

/// Package.json content of a description: the collected mirror when present,
/// otherwise read from the described file.
pub(crate) fn description_file_json(
  description: &rspack_loader_runner::DescriptionData,
) -> Option<serde_json::Value> {
  let json = description.json();
  if !json.is_null() {
    return Some(json.clone());
  }
  read_description_file_json(description.json_path())
}

impl From<&rspack_core::ResourceData> for JsResourceData {
  fn from(value: &rspack_core::ResourceData) -> Self {
    Self {
      resource: value.resource().to_owned(),
      path: value.path().map(|p| p.as_str().to_string()),
      fragment: value.fragment().map(|r| r.to_owned()),
      query: value.query().map(|r| r.to_owned()),
      description_file_data: value.description().and_then(description_file_json),
      description_file_path: value
        .description()
        .map(|data| data.path().to_string_lossy().into_owned()),
    }
  }
}

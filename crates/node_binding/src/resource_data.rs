use std::sync::Arc;

use napi::bindgen_prelude::{FromNapiRef, FromNapiValue};
use napi_derive::napi;
use rspack_core::ResourceData;

// Converting the descriptionFileData property to a JSObject may become a performance bottleneck.
// Therefore, ResourceData is exposed as a class instance with properties accessed lazily.
#[napi]
pub struct ReadonlyResourceData {
  i: Arc<rspack_core::ResourceData>,
}

#[napi]
impl ReadonlyResourceData {
  #[napi(getter)]
  pub fn resource(&self) -> &str {
    self.i.resource.as_str()
  }

  #[napi(getter)]
  pub fn path(&self) -> Option<&str> {
    self.i.resource_path.as_ref().map(|path| path.as_str())
  }

  #[napi(getter)]
  pub fn query(&self) -> Option<&str> {
    self.i.resource_query.as_deref()
  }

  #[napi(getter)]
  pub fn fragment(&self) -> Option<&str> {
    self.i.resource_fragment.as_deref()
  }

  #[napi(getter)]
  pub fn description_file_data(&mut self) -> Option<&serde_json::Value> {
    self.i.resource_description.as_ref().map(|data| data.json())
  }

  #[napi(getter)]
  pub fn description_file_path(&mut self) -> Option<String> {
    self
      .i
      .resource_description
      .as_ref()
      .map(|data| data.path().to_string_lossy().to_string())
  }
}

impl FromNapiValue for ReadonlyResourceData {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let resource_data: &ReadonlyResourceData = FromNapiRef::from_napi_ref(env, napi_val)?;
    Ok(resource_data.i.clone().into())
  }
}

impl From<Arc<rspack_core::ResourceData>> for ReadonlyResourceData {
  fn from(value: Arc<rspack_core::ResourceData>) -> Self {
    Self { i: value }
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

impl From<ResourceData> for JsResourceData {
  fn from(value: ResourceData) -> Self {
    let (description_file_path, description_file_data) = value
      .resource_description
      .map(|data| data.into_parts())
      .unzip();
    Self {
      resource: value.resource,
      path: value.resource_path.map(|p| p.as_str().to_string()),
      query: value.resource_query,
      fragment: value.resource_fragment,
      description_file_data: description_file_data.map(std::sync::Arc::unwrap_or_clone),
      description_file_path: description_file_path.map(|path| path.to_string_lossy().into_owned()),
    }
  }
}

impl From<&ResourceData> for JsResourceData {
  fn from(value: &ResourceData) -> Self {
    Self {
      resource: value.resource.to_owned(),
      path: value.resource_path.as_ref().map(|p| p.as_str().to_string()),
      fragment: value.resource_fragment.as_ref().map(|r| r.to_owned()),
      query: value.resource_query.as_ref().map(|r| r.to_owned()),
      description_file_data: value
        .resource_description
        .as_ref()
        .map(|data| data.json().to_owned()),
      description_file_path: value
        .resource_description
        .as_ref()
        .map(|data| data.path().to_string_lossy().into_owned()),
    }
  }
}

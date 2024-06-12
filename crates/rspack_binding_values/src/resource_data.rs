use napi_derive::napi;
use rspack_core::ResourceData;

#[napi(object)]
pub struct JsResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub path: String,
  /// Resource query with `?` prefix
  pub query: Option<String>,
  /// Resource fragment with `#` prefix
  pub fragment: Option<String>,
}

impl From<ResourceData> for JsResourceData {
  fn from(value: ResourceData) -> Self {
    Self {
      resource: value.resource,
      path: value.resource_path.to_string_lossy().to_string(),
      query: value.resource_query,
      fragment: value.resource_fragment,
    }
  }
}

impl From<&ResourceData> for JsResourceData {
  fn from(value: &ResourceData) -> Self {
    Self {
      resource: value.resource.to_owned(),
      path: value.resource_path.to_string_lossy().to_string(),
      fragment: value.resource_fragment.as_ref().map(|r| r.to_owned()),
      query: value.resource_query.as_ref().map(|r| r.to_owned()),
    }
  }
}

use rspack_core::{NormalModuleFactoryResolveForSchemeArgs, ResourceData};

#[napi(object)]
pub struct SchemeAndJsResourceData {
  pub resource_data: JsResourceData,
  pub scheme: String,
}

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

impl From<NormalModuleFactoryResolveForSchemeArgs> for SchemeAndJsResourceData {
  fn from(value: NormalModuleFactoryResolveForSchemeArgs) -> Self {
    Self {
      resource_data: value.resource.into(),
      scheme: value.scheme,
    }
  }
}

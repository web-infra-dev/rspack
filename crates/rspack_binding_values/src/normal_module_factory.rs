use napi_derive::napi;
use rspack_core::{NormalModuleCreateData, ResourceData};

#[napi(object)]
pub struct JsResolveForSchemeArgs {
  pub resource_data: JsResourceData,
  pub scheme: String,
}

pub type JsResolveForSchemeOutput = (Option<bool>, JsResourceData);

#[napi(object)]
pub struct JsBeforeResolveArgs {
  pub request: String,
  pub context: String,
}

pub type JsBeforeResolveOutput = (Option<bool>, JsBeforeResolveArgs);

#[napi(object)]
pub struct JsCreateData {
  pub request: String,
  pub user_request: String,
  pub resource: String,
}

#[napi(object)]
pub struct JsAfterResolveData {
  pub request: String,
  pub context: String,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub create_data: Option<JsCreateData>,
}

pub type JsAfterResolveOutput = (Option<bool>, Option<JsCreateData>);

#[napi(object)]
pub struct JsNormalModuleFactoryCreateModuleArgs {
  pub dependency_type: String,
  pub raw_request: String,
  pub resource_resolve_data: JsResourceData,
  pub context: String,
  pub match_resource: Option<String>,
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

impl From<ResourceData> for JsResolveForSchemeArgs {
  fn from(value: ResourceData) -> Self {
    Self {
      scheme: value.get_scheme().to_string(),
      resource_data: value.into(),
    }
  }
}

impl From<&NormalModuleCreateData> for JsCreateData {
  fn from(value: &NormalModuleCreateData) -> Self {
    Self {
      request: value.request.to_owned(),
      user_request: value.user_request.to_owned(),
      resource: value
        .resource_resolve_data
        .resource_path
        .to_string_lossy()
        .to_string(),
    }
  }
}

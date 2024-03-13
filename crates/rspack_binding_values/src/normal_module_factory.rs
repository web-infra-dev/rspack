use napi_derive::napi;
use rspack_core::{
  BeforeResolveArgs, NormalModuleAfterResolveArgs, NormalModuleCreateData, ResourceData,
};

#[napi(object)]
pub struct JsResolveForSchemeInput {
  pub resource_data: JsResourceData,
  pub scheme: String,
}

#[napi(object)]
pub struct JsResolveForSchemeResult {
  pub resource_data: JsResourceData,
  pub stop: bool,
}

#[napi(object)]
pub struct JsBeforeResolveArgs {
  pub request: String,
  pub context: String,
}

#[napi(object)]
pub struct AfterResolveData {
  pub request: String,
  pub context: String,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub factory_meta: FactoryMeta,
}

#[napi(object)]
pub struct CreateModuleData {
  pub dependency_type: String,
  pub resolve_data_request: String,
  pub resource_resolve_data: JsResourceData,
  pub context: String,
}

#[napi(object)]
pub struct FactoryMeta {
  pub side_effect_free: Option<bool>,
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

impl From<ResourceData> for JsResolveForSchemeInput {
  fn from(value: ResourceData) -> Self {
    Self {
      scheme: value.get_scheme().to_string(),
      resource_data: value.into(),
    }
  }
}

impl From<&mut NormalModuleCreateData<'_>> for CreateModuleData {
  fn from(value: &mut NormalModuleCreateData) -> Self {
    Self {
      context: value.context.to_string(),
      dependency_type: value.dependency_type.to_string(),
      resolve_data_request: value.resolve_data_request.into(),
      resource_resolve_data: value.resource_resolve_data.clone().into(),
    }
  }
}

impl From<BeforeResolveArgs> for JsBeforeResolveArgs {
  fn from(value: BeforeResolveArgs) -> Self {
    Self {
      context: value.context,
      request: value.request,
    }
  }
}

impl From<&NormalModuleAfterResolveArgs<'_>> for AfterResolveData {
  fn from(value: &NormalModuleAfterResolveArgs) -> Self {
    Self {
      context: value.context.to_owned(),
      request: value.request.to_string(),
      file_dependencies: value
        .file_dependencies
        .clone()
        .into_iter()
        .map(|item| item.to_string_lossy().to_string())
        .collect::<Vec<_>>(),
      context_dependencies: value
        .context_dependencies
        .clone()
        .into_iter()
        .map(|item| item.to_string_lossy().to_string())
        .collect::<Vec<_>>(),
      missing_dependencies: value
        .context_dependencies
        .clone()
        .into_iter()
        .map(|item| item.to_string_lossy().to_string())
        .collect::<Vec<_>>(),
      factory_meta: FactoryMeta {
        side_effect_free: value.factory_meta.side_effect_free,
      },
    }
  }
}

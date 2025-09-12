use napi::{
  Env,
  bindgen_prelude::{Object, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{ModuleFactoryCreateData, NormalModuleCreateData, ResourceData, parse_resource};
use serde::Serialize;

use crate::resource_data::JsResourceData;

#[napi(object)]
pub struct JsResolveForSchemeArgs {
  pub resource_data: JsResourceData,
  pub scheme: String,
}

pub type JsResolveForSchemeOutput = (Option<bool>, JsResourceData);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct ContextInfo {
  pub issuer: String,
  pub issuer_layer: Option<String>,
}

impl ToNapiValue for &ContextInfo {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    unsafe {
      let env_wrapper = Env::from(env);
      let mut obj = Object::new(&env_wrapper)?;
      obj.set("issuer", &val.issuer)?;
      if let Some(issuer_layer) = &val.issuer_layer {
        obj.set("issuerLayer", issuer_layer)?;
      }
      ToNapiValue::to_napi_value(env, obj)
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct JsCreateData {
  pub request: String,
  pub user_request: String,
  pub resource: String,
}

impl From<&NormalModuleCreateData> for JsCreateData {
  fn from(value: &NormalModuleCreateData) -> Self {
    Self {
      request: value.request.to_owned(),
      user_request: value.user_request.to_owned(),
      resource: value.resource_resolve_data.resource().to_owned(),
    }
  }
}

impl JsCreateData {
  pub fn update_nmf_data(self, create_data: &mut NormalModuleCreateData) {
    create_data.request = self.request;
    create_data.user_request = self.user_request;
    if create_data.resource_resolve_data.resource() != self.resource {
      create_data
        .resource_resolve_data
        .update_resource_data(self.resource);
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct JsResolveData {
  pub request: String,
  pub context: String,
  pub context_info: ContextInfo,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub create_data: Option<JsCreateData>,
}

impl JsResolveData {
  pub fn from_nmf_data(
    data: &ModuleFactoryCreateData,
    create_data: Option<&NormalModuleCreateData>,
  ) -> Self {
    JsResolveData {
      request: data.request.to_string(),
      context: data.context.to_string(),
      context_info: ContextInfo {
        issuer: data
          .issuer
          .as_ref()
          .map(|issuer| issuer.to_string())
          .unwrap_or_default(),
        issuer_layer: data.issuer_layer.clone(),
      },
      file_dependencies: data
        .file_dependencies
        .iter()
        .map(|item| item.to_string_lossy().into_owned())
        .collect::<Vec<_>>(),
      context_dependencies: data
        .context_dependencies
        .iter()
        .map(|item| item.to_string_lossy().into_owned())
        .collect::<Vec<_>>(),
      missing_dependencies: data
        .missing_dependencies
        .iter()
        .map(|item| item.to_string_lossy().into_owned())
        .collect::<Vec<_>>(),
      create_data: create_data.map(|create_data| JsCreateData {
        request: create_data.request.to_owned(),
        user_request: create_data.user_request.to_owned(),
        resource: create_data.resource_resolve_data.resource().to_owned(),
      }),
    }
  }

  pub fn update_nmf_data(
    self,
    data: &mut ModuleFactoryCreateData,
    create_data: Option<&mut NormalModuleCreateData>,
  ) {
    // only supports update request for now
    data.request = self.request;
    data.context = self.context.into();
    if let Some(new_data) = self.create_data
      && let Some(create_data) = create_data
    {
      new_data.update_nmf_data(create_data);
    }
  }
}

#[napi(object)]
pub struct JsNormalModuleFactoryCreateModuleArgs {
  pub dependency_type: String,
  pub raw_request: String,
  pub resource_resolve_data: JsResourceData,
  pub context: String,
  pub match_resource: Option<String>,
}

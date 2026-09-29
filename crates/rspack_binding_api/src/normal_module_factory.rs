use napi::{
  Env, JsValue,
  bindgen_prelude::{JavaScriptClassExt, Object, ToNapiValue},
  sys::napi_value,
};
use napi_derive::napi;
use rspack_core::{ModuleFactoryCreateData, NormalModuleCreateData, ResourceData, parse_resource};
use rspack_paths::InternedPath;
use rustc_hash::FxHashMap as HashMap;
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
      request: value.request.clone(),
      user_request: value.user_request.clone(),
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

#[derive(Debug)]
#[napi]
pub struct JsPathDependencies {
  // Own the paths independently of the factory and compiler lifetimes.
  file_dependencies: Vec<InternedPath>,
  context_dependencies: Vec<InternedPath>,
  missing_dependencies: Vec<InternedPath>,
}

#[napi]
impl JsPathDependencies {
  #[napi(getter)]
  pub fn file_dependencies(&self) -> Vec<String> {
    self
      .file_dependencies
      .iter()
      .map(|path| path.to_string_lossy().into_owned())
      .collect()
  }

  #[napi(getter)]
  pub fn context_dependencies(&self) -> Vec<String> {
    self
      .context_dependencies
      .iter()
      .map(|path| path.to_string_lossy().into_owned())
      .collect()
  }

  #[napi(getter)]
  pub fn missing_dependencies(&self) -> Vec<String> {
    self
      .missing_dependencies
      .iter()
      .map(|path| path.to_string_lossy().into_owned())
      .collect()
  }
}

pub struct JsResolveData {
  request: String,
  context: String,
  context_info: ContextInfo,
  attributes: Option<HashMap<String, String>>,
  create_data: Option<JsCreateData>,
  dependencies: JsPathDependencies,
}

impl JsResolveData {
  pub fn from_nmf_data(
    data: &ModuleFactoryCreateData,
    create_data: Option<&NormalModuleCreateData>,
  ) -> Self {
    Self {
      request: data.request.clone(),
      context: data.context.to_string(),
      context_info: ContextInfo {
        issuer: data
          .issuer
          .as_ref()
          .map(|issuer| issuer.to_string())
          .unwrap_or_default(),
        issuer_layer: data.issuer_layer.clone(),
      },
      attributes: data
        .dependencies
        .first()
        .and_then(|dependency| dependency.get_attributes())
        .map(|attributes| {
          attributes
            .iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect()
        }),
      create_data: create_data.map(|create_data| JsCreateData {
        request: create_data.request.clone(),
        user_request: create_data.user_request.clone(),
        resource: create_data.resource_resolve_data.resource().to_owned(),
      }),
      dependencies: JsPathDependencies {
        file_dependencies: data.file_dependencies.iter().cloned().collect(),
        context_dependencies: data.context_dependencies.iter().cloned().collect(),
        missing_dependencies: data.missing_dependencies.iter().cloned().collect(),
      },
    }
  }
}

impl ToNapiValue for JsResolveData {
  unsafe fn to_napi_value(env: napi::sys::napi_env, val: Self) -> napi::Result<napi_value> {
    let env = Env::from_raw(env);
    let instance = val.dependencies.into_instance(&env)?;
    let mut object = instance.as_object(&env);
    // Keep common fields as JS-owned data properties, including mutable createData.
    object.set("request", val.request)?;
    object.set("context", val.context)?;
    object.set("contextInfo", val.context_info)?;
    if let Some(attributes) = val.attributes {
      object.set("attributes", attributes)?;
    }
    if let Some(create_data) = val.create_data {
      object.set("createData", create_data)?;
    }
    Ok(object.raw())
  }
}

#[napi(object, object_to_js = false)]
pub struct JsResolveDataUpdate {
  pub request: String,
  pub context: String,
  pub create_data: Option<JsCreateData>,
}

impl JsResolveDataUpdate {
  pub fn update_nmf_data(
    self,
    data: &mut ModuleFactoryCreateData,
    create_data: Option<&mut NormalModuleCreateData>,
  ) {
    // Other fields are snapshots and are not written back to the factory.
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

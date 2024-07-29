use napi_derive::napi;
use rspack_core::{CompilerModuleContext, Module};
use rspack_napi::napi::bindgen_prelude::*;

use super::{JsCompatSource, ToJsCompatSource};
use crate::{JsChunk, JsCodegenerationResults};

#[derive(Default)]
#[napi(object)]
pub struct JsFactoryMeta {
  pub side_effect_free: Option<bool>,
}

#[derive(Default)]
#[napi(object)]
pub struct JsModule {
  pub context: Option<String>,
  pub original_source: Option<JsCompatSource>,
  pub resource: Option<String>,
  pub module_identifier: String,
  pub name_for_condition: Option<String>,
  pub request: Option<String>,
  pub user_request: Option<String>,
  pub raw_request: Option<String>,
  pub factory_meta: Option<JsFactoryMeta>,
  pub r#type: String,
  pub layer: Option<String>,
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsModule>;
}

impl ToJsModule for dyn Module {
  fn to_js_module(&self) -> Result<JsModule> {
    let original_source = || {
      self
        .original_source()
        .and_then(|source| source.to_js_compat_source().ok())
    };
    let name_for_condition = || self.name_for_condition().map(|s| s.to_string());
    let module_identifier = || self.identifier().to_string();
    let context = || self.get_context().map(|c| c.to_string());
    let module_type = || self.module_type().to_string();
    let module_layer = || self.get_layer().cloned();

    self
      .try_as_normal_module()
      .map(|normal_module| JsModule {
        context: context(),
        original_source: original_source(),
        resource: Some(normal_module.resource_resolved_data().resource.to_string()),
        r#type: module_type(),
        layer: module_layer(),
        module_identifier: module_identifier(),
        name_for_condition: name_for_condition(),
        request: Some(normal_module.request().to_string()),
        user_request: Some(normal_module.user_request().to_string()),
        raw_request: Some(normal_module.raw_request().to_string()),
        factory_meta: normal_module
          .factory_meta()
          .map(|factory_meta| JsFactoryMeta {
            side_effect_free: factory_meta.side_effect_free,
          }),
      })
      .or_else(|_| {
        self.try_as_raw_module().map(|_| JsModule {
          context: context(),
          r#type: module_type(),
          layer: module_layer(),
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          user_request: None,
          request: None,
          factory_meta: None,
        })
      })
      .or_else(|_| {
        self.try_as_context_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          r#type: module_type(),
          layer: module_layer(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          user_request: None,
          request: None,
          factory_meta: None,
        })
      })
      .or_else(|_| {
        self.try_as_external_module().map(|_| JsModule {
          context: context(),
          original_source: original_source(),
          r#type: module_type(),
          layer: module_layer(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          raw_request: None,
          user_request: None,
          request: None,
          factory_meta: None,
        })
      })
      .or_else(|_| {
        Ok(JsModule {
          context: context(),
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
          layer: module_layer(),
          r#type: module_type(),
          ..Default::default()
        })
      })
  }
}

impl ToJsModule for CompilerModuleContext {
  fn to_js_module(&self) -> Result<JsModule> {
    let module = JsModule {
      context: self.context.as_ref().map(|c| c.to_string()),
      module_identifier: self.module_identifier.to_string(),
      name_for_condition: self.name_for_condition.clone(),
      r#type: self.r#type.to_string(),
      layer: self.layer.clone(),
      resource: self.resource_data.as_ref().map(|r| r.resource.to_string()),
      original_source: None,
      request: self.request.clone(),
      user_request: self.user_request.clone(),
      raw_request: self.raw_request.clone(),
      factory_meta: self.factory_meta.as_ref().map(|fm| JsFactoryMeta {
        side_effect_free: fm.side_effect_free,
      }),
    };
    Ok(module)
  }
}

#[napi(object)]
pub struct JsExecuteModuleArg {
  pub entry: String,
  pub runtime_modules: Vec<String>,
  pub codegen_results: JsCodegenerationResults,
  pub id: u32,
}

#[derive(Default)]
#[napi(object)]
pub struct JsRuntimeModule {
  pub source: Option<JsCompatSource>,
  pub module_identifier: String,
  pub constructor_name: String,
  pub name: String,
}

#[napi(object)]
pub struct JsRuntimeModuleArg {
  pub module: JsRuntimeModule,
  pub chunk: JsChunk,
}

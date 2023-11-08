use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::Module;
use rspack_identifier::Identifiable;

use super::{JsCompatSource, ToJsCompatSource};

#[napi(object)]
pub struct JsModule {
  pub original_source: Option<JsCompatSource>,
  pub resource: Option<String>,
  pub module_identifier: String,
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsModule>;
}

impl ToJsModule for dyn Module + '_ {
  fn to_js_module(&self) -> Result<JsModule> {
    let original_source = self
      .original_source()
      .and_then(|source| source.to_js_compat_source().ok());

    self
      .try_as_normal_module()
      .map(|normal_module| JsModule {
        original_source: original_source.clone(),
        resource: Some(
          normal_module
            .resource_resolved_data()
            .resource_path
            .to_string_lossy()
            .to_string(),
        ),
        module_identifier: normal_module.identifier().to_string(),
      })
      .or_else(|_| {
        self.try_as_raw_module().map(|module| JsModule {
          original_source: original_source.clone(),
          resource: None,
          module_identifier: module.identifier().to_string(),
        })
      })
      .or_else(|_| {
        self.try_as_context_module().map(|module| JsModule {
          original_source: original_source.clone(),
          resource: None,
          module_identifier: module.identifier().to_string(),
        })
      })
      .or_else(|_| {
        self.try_as_external_module().map(|module| JsModule {
          original_source: original_source.clone(),
          resource: None,
          module_identifier: module.identifier().to_string(),
        })
      })
      .map_err(|_| napi::Error::from_reason("Failed to convert module to JsModule"))
  }
}

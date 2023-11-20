use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::Module;

use super::{JsCompatSource, ToJsCompatSource};

#[napi(object)]
pub struct JsModule {
  pub original_source: Option<JsCompatSource>,
  pub resource: Option<String>,
  pub module_identifier: String,
  pub name_for_condition: Option<String>,
}

pub trait ToJsModule {
  fn to_js_module(&self) -> Result<JsModule>;
}

impl ToJsModule for dyn Module + '_ {
  fn to_js_module(&self) -> Result<JsModule> {
    let original_source = || {
      self
        .original_source()
        .and_then(|source| source.to_js_compat_source().ok())
    };
    let name_for_condition = || self.name_for_condition().map(|s| s.to_string());
    let module_identifier = || self.identifier().to_string();

    self
      .try_as_normal_module()
      .map(|normal_module| JsModule {
        original_source: original_source(),
        resource: Some(
          normal_module
            .resource_resolved_data()
            .resource_path
            .to_string_lossy()
            .to_string(),
        ),
        module_identifier: module_identifier(),
        name_for_condition: name_for_condition(),
      })
      .or_else(|_| {
        self.try_as_raw_module().map(|_| JsModule {
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
        })
      })
      .or_else(|_| {
        self.try_as_context_module().map(|_| JsModule {
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
        })
      })
      .or_else(|_| {
        self.try_as_external_module().map(|_| JsModule {
          original_source: original_source(),
          resource: None,
          module_identifier: module_identifier(),
          name_for_condition: name_for_condition(),
        })
      })
      .map_err(|_| napi::Error::from_reason("Failed to convert module to JsModule"))
  }
}

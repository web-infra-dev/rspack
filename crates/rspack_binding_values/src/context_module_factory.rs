use napi::bindgen_prelude::Either;
use napi_derive::napi;

use crate::RawRegex;

#[napi(object)]
pub struct JsContextModuleFactoryBeforeResolveData {
  pub context: String,
  pub request: Option<String>,
}

pub type JsContextModuleFactoryBeforeResolveResult =
  Either<bool, JsContextModuleFactoryBeforeResolveData>;

#[napi(object)]
pub struct JsContextModuleFactoryAfterResolveData {
  pub resource: String,
  pub context: String,
  pub request: String,
  pub reg_exp: Option<RawRegex>,
}

pub type JsContextModuleFactoryAfterResolveResult =
  Either<bool, JsContextModuleFactoryAfterResolveData>;

#[napi(object)]
pub struct JsAlternativeRequest {
  pub context: String,
  pub request: String,
}

impl From<rspack_core::AlternativeRequest> for JsAlternativeRequest {
  fn from(value: rspack_core::AlternativeRequest) -> Self {
    Self {
      context: value.context,
      request: value.request,
    }
  }
}

#[napi(object)]
pub struct JsContextModuleOptions {
  pub addon: String,
  pub category: String,
  pub mode: String,
  pub recursive: bool,
  pub request: String,
  pub resource: String,
  pub resource_query: String,
  pub resource_fragment: String,
}

impl From<rspack_core::ContextModuleOptions> for JsContextModuleOptions {
  fn from(value: rspack_core::ContextModuleOptions) -> Self {
    Self {
      addon: value.addon,
      category: value.context_options.category.as_str().to_owned(),
      mode: value.context_options.mode.as_str().to_owned(),
      recursive: value.context_options.recursive,
      request: value.context_options.request,
      resource: value.resource.as_str().to_owned(),
      resource_query: value.resource_query,
      resource_fragment: value.resource_fragment,
    }
  }
}

#[napi(object)]
pub struct JsAlternativeRequestsArgs {
  pub requests: Vec<JsAlternativeRequest>,
  pub options: JsContextModuleOptions,
}

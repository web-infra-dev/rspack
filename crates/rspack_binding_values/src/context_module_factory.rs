use napi::bindgen_prelude::Either;
use napi_derive::napi;

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
  pub reg_exp: Option<String>,
}

pub type JsContextModuleFactoryAfterResolveResult =
  Either<bool, JsContextModuleFactoryAfterResolveData>;

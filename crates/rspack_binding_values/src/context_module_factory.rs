use napi::bindgen_prelude::Either;
use napi_derive::napi;

use crate::RawRegex;

#[napi(object)]
pub struct JsContextModuleFactoryBeforeResolveData {
  pub context: String,
  pub request: String,
  pub reg_exp: Option<RawRegex>,
  pub recursive: bool,
}

pub type JsContextModuleFactoryBeforeResolveResult =
  Either<bool, JsContextModuleFactoryBeforeResolveData>;

#[napi(object)]
pub struct JsContextModuleFactoryAfterResolveData {
  pub resource: String,
  pub context: String,
  pub request: String,
  pub reg_exp: Option<RawRegex>,
  pub recursive: bool,
}

pub type JsContextModuleFactoryAfterResolveResult =
  Either<bool, JsContextModuleFactoryAfterResolveData>;

use napi_derive::napi;

#[napi(object)]
pub struct JsContextModuleFactoryAfterResolveArgs {
  pub resource: String,
  pub reg_exp: Option<String>,
}

pub type JsContextModuleFactoryAfterResolveResult =
  (Option<bool>, JsContextModuleFactoryAfterResolveArgs);

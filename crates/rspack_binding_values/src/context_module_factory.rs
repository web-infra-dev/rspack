use napi_derive::napi;

#[napi(object)]
pub struct JsContextModuleFactoryAfterResolveResult {
  pub resource: String,
  pub context: String,
  pub request: String,
  pub reg_exp: Option<String>,
}

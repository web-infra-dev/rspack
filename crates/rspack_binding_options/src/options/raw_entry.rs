use napi_derive::napi;
use rspack_binding_values::JsEntryOptions;

#[derive(Debug)]
#[napi(object)]
pub struct RawEntryPluginOptions {
  pub context: String,
  pub entry: String,
  pub options: JsEntryOptions,
}

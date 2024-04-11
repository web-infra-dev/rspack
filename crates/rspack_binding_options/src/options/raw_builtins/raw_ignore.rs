use derivative::Derivative;
use napi_derive::napi;
use rspack_napi::regexp::{JsRegExp, JsRegExpExt};
use rspack_plugin_ignore::IgnorePluginOptions;

#[derive(Derivative)]
#[derivative(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawIgnorePluginOptions {
  #[napi(ts_type = "RegExp")]
  pub resource_reg_exp: JsRegExp,
  #[napi(ts_type = "RegExp")]
  pub context_reg_exp: Option<JsRegExp>,
}

impl From<RawIgnorePluginOptions> for IgnorePluginOptions {
  fn from(value: RawIgnorePluginOptions) -> Self {
    Self {
      resource_reg_exp: value.resource_reg_exp.to_rspack_regex(),
      context_reg_exp: value
        .context_reg_exp
        .map(|context_reg_exp| context_reg_exp.to_rspack_regex()),
    }
  }
}

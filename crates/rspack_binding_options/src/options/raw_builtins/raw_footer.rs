use napi_derive::napi;
use rspack_plugin_footer::FooterPluginOptions;

#[derive(Debug)]
#[napi(object)]
pub struct RawFooterPluginOptions {
  pub footer: String,
}

impl From<RawFooterPluginOptions> for FooterPluginOptions {
  fn from(value: RawFooterPluginOptions) -> Self {
    FooterPluginOptions {
      footer: value.footer,
    }
  }
}

use napi_derive::napi;
use rspack_plugin_progress::ProgressPluginOptions;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawProgressPluginOptions {
  pub prefix: String,
  pub profile: bool,
  // indicatif::ProgressBar template
  pub template: String,
  pub tick_strings: Option<Vec<String>>,
  pub progress_chars: String,
}

impl From<RawProgressPluginOptions> for ProgressPluginOptions {
  fn from(value: RawProgressPluginOptions) -> Self {
    Self {
      prefix: value.prefix,
      profile: value.profile,
      template: value.template,
      tick_strings: value.tick_strings,
      progress_chars: value.progress_chars,
    }
  }
}

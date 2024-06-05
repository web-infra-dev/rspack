use napi::Either;
use napi_derive::napi;
use rspack_plugin_progress::ProgressPluginOptions;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawProgressPluginOptions {
  pub prefix: String,
  pub profile: bool,
  pub template: String,
  pub tick: Option<Either<String, Vec<String>>>,
  pub progress_chars: String,
}

impl From<RawProgressPluginOptions> for ProgressPluginOptions {
  fn from(value: RawProgressPluginOptions) -> Self {
    Self {
      prefix: value.prefix,
      profile: value.profile,
      template: value.template,
      progress_chars: value.progress_chars,
      tick_strings: value.tick.map(|tick| match tick {
        Either::A(str) => str.chars().map(|c| c.to_string()).collect(),
        Either::B(vec) => vec,
      }),
    }
  }
}

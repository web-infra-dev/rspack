use napi::Either;
use napi_derive::napi;
use rspack_plugin_runtime::BundlerInfoMode;
use rustc_hash::FxHashSet;

type RawBundlerInfoMode = Either<String, Vec<String>>;
pub struct RawBundlerInfoModeWrapper(pub RawBundlerInfoMode);

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawBundlerInfoPluginOptions {
  pub version: String,
  #[napi(ts_type = "string | string[]")]
  pub mode: RawBundlerInfoMode,
}

impl From<RawBundlerInfoModeWrapper> for BundlerInfoMode {
  fn from(x: RawBundlerInfoModeWrapper) -> Self {
    match x.0 {
      Either::A(v) => match v.as_str() {
        "all" => BundlerInfoMode::All,
        "auto" => BundlerInfoMode::Auto,
        _ => BundlerInfoMode::Auto,
      },
      Either::B(v) => BundlerInfoMode::Partial(v.into_iter().collect::<FxHashSet<String>>()),
    }
  }
}

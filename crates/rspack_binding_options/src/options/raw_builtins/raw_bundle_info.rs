use napi::Either;
use napi_derive::napi;
use rspack_plugin_runtime::BundlerInfoForceMode;
use rustc_hash::FxHashSet;

type RawBundlerInfoMode = Either<bool, Vec<String>>;
pub struct RawBundlerInfoModeWrapper(pub RawBundlerInfoMode);

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawBundlerInfoPluginOptions {
  pub version: String,
  pub bundler: String,
  #[napi(ts_type = "boolean | string[]")]
  pub force: RawBundlerInfoMode,
}

impl From<RawBundlerInfoModeWrapper> for BundlerInfoForceMode {
  fn from(x: RawBundlerInfoModeWrapper) -> Self {
    match x.0 {
      Either::A(v) => {
        if v {
          BundlerInfoForceMode::All
        } else {
          BundlerInfoForceMode::Auto
        }
      }
      Either::B(v) => BundlerInfoForceMode::Partial(v.into_iter().collect::<FxHashSet<String>>()),
    }
  }
}

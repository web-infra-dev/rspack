use napi_derive::napi;
use rspack_core::RspackFuture;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawExperiments {
  pub layers: bool,
  pub top_level_await: bool,
  pub incremental: Option<RawIncremental>,
  pub rspack_future: RawRspackFuture,
}

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawIncremental {
  pub make: bool,
  pub emit_assets: bool,
  pub infer_async_modules: bool,
  pub provided_exports: bool,
  pub collect_modules_diagnostics: bool,
  pub module_hashes: bool,
  pub module_codegen: bool,
  pub module_runtime_requirements: bool,
}

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug, Default)]
#[napi(object)]
pub struct RawRspackFuture {}

impl From<RawRspackFuture> for RspackFuture {
  fn from(_value: RawRspackFuture) -> Self {
    Self {}
  }
}

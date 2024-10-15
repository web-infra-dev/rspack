use napi_derive::napi;
use rspack_core::{Incremental, RspackFuture};

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
  pub collect_module_diagnostics: bool,
  pub module_hashes: bool,
  pub module_codegen: bool,
  pub module_runtime_requirements: bool,
}

impl From<RawIncremental> for Incremental {
  fn from(value: RawIncremental) -> Self {
    Self::Enabled {
      make: value.make,
      emit_assets: value.emit_assets,
      infer_async_modules: value.infer_async_modules,
      provided_exports: value.provided_exports,
      collect_module_diagnostics: value.collect_module_diagnostics,
      module_hashes: value.module_hashes,
      module_codegen: value.module_codegen,
      module_runtime_requirements: value.module_runtime_requirements,
    }
  }
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

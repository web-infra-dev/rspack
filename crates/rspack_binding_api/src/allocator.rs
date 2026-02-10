use napi::{Env, bindgen_prelude::ToNapiValue, sys::napi_env};
use rspack_core::BindingCell;

use crate::{
  asset::AssetInfo,
  build_info::Assets,
  compilation::{CodeGenerationResult, CodeGenerationResults, Sources},
};

pub(crate) struct NapiAllocatorImpl;

impl NapiAllocatorImpl {
  pub fn new(_env: Env) -> Self {
    Self
  }
}

impl rspack_core::NapiAllocator for NapiAllocatorImpl {
  fn allocate_asset_info(
    &self,
    env: napi_env,
    val: &BindingCell<rspack_core::AssetInfo>,
  ) -> napi::Result<napi::sys::napi_value> {
    let asset_info: AssetInfo = (**val).clone().into();
    unsafe { ToNapiValue::to_napi_value(env, asset_info) }
  }

  fn allocate_code_generation_result(
    &self,
    env: napi_env,
    val: &BindingCell<rspack_core::CodeGenerationResult>,
  ) -> napi::Result<napi::sys::napi_value> {
    let code_generation_result = CodeGenerationResult::new(val.reflector());
    unsafe { ToNapiValue::to_napi_value(env, code_generation_result) }
  }

  fn allocate_sources(
    &self,
    env: napi_env,
    val: &BindingCell<
      rustc_hash::FxHashMap<rspack_core::SourceType, rspack_core::rspack_sources::BoxSource>,
    >,
  ) -> napi::Result<napi::sys::napi_value> {
    let sources = Sources::new(val.reflector());
    unsafe { ToNapiValue::to_napi_value(env, sources) }
  }

  fn allocate_code_generation_results(
    &self,
    env: napi_env,
    val: &BindingCell<rspack_core::CodeGenerationResults>,
  ) -> napi::Result<napi::sys::napi_value> {
    let code_generation_results = CodeGenerationResults::new(val.reflector());
    unsafe { ToNapiValue::to_napi_value(env, code_generation_results) }
  }

  fn allocate_assets(
    &self,
    env: napi_env,
    val: &BindingCell<rustc_hash::FxHashMap<String, rspack_core::CompilationAsset>>,
  ) -> napi::Result<napi::sys::napi_value> {
    let assets = Assets::new(val.reflector());
    unsafe { ToNapiValue::to_napi_value(env, assets) }
  }
}

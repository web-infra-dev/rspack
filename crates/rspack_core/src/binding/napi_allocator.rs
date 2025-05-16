use napi::sys::{napi_env, napi_value};
use once_cell::sync::OnceCell;
use rspack_sources::BoxSource;
use rustc_hash::FxHashMap;

use super::BindingCell;
use crate::{AssetInfo, CodeGenerationResult, CodeGenerationResults, CompilationAsset, SourceType};

thread_local! {
  static NAPI_ALLOCATOR: OnceCell<Box<dyn NapiAllocator>> = OnceCell::default();
}

pub trait NapiAllocator {
  fn allocate_asset_info(
    &self,
    env: napi_env,
    val: &BindingCell<AssetInfo>,
  ) -> napi::Result<napi_value>;
  fn allocate_code_generation_results(
    &self,
    env: napi_env,
    val: &BindingCell<CodeGenerationResults>,
  ) -> napi::Result<napi_value>;
  fn allocate_code_generation_result(
    &self,
    env: napi_env,
    val: &BindingCell<CodeGenerationResult>,
  ) -> napi::Result<napi_value>;
  fn allocate_sources(
    &self,
    env: napi_env,
    val: &BindingCell<FxHashMap<SourceType, BoxSource>>,
  ) -> napi::Result<napi_value>;
  fn allocate_assets(
    &self,
    env: napi_env,
    val: &BindingCell<FxHashMap<String, CompilationAsset>>,
  ) -> napi::Result<napi_value>;
}

pub fn with_thread_local_allocator<T>(
  f: impl FnOnce(&Box<dyn NapiAllocator>) -> napi::Result<T>,
) -> napi::Result<T> {
  NAPI_ALLOCATOR.with(|once_cell| match once_cell.get() {
    Some(allocator) => f(allocator),
    None => Err(napi::Error::new(
      napi::Status::GenericFailure,
      "Allocator is not set in current thread",
    )),
  })
}

pub fn set_thread_local_allocator(allocator: Box<dyn NapiAllocator>) {
  NAPI_ALLOCATOR.with(|once_cell| {
    once_cell.get_or_init(|| allocator);
  })
}

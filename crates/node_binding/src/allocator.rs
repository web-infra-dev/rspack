use napi::{bindgen_prelude::ToNapiValue, sys::napi_env, Env};
use rspack_core::BindingCell;

use crate::AssetInfo;

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
}

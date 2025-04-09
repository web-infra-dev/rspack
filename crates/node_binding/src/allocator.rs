use napi::{bindgen_prelude::ToNapiValue, sys::napi_env, Env};
use rspack_core::WeakBindingCell;

use crate::{
  entries::{EntryDataDTO, EntryOptionsDTO},
  AssetInfo,
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
    val: WeakBindingCell<rspack_core::AssetInfo>,
  ) -> napi::Result<napi::sys::napi_value> {
    let heap = val.upgrade().ok_or_else(|| {
      napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to upgrade weak reference to heap",
      )
    })?;
    let asset_info: AssetInfo = (*heap).clone().into();
    unsafe { ToNapiValue::to_napi_value(env, asset_info) }
  }

  fn allocate_entry_data(
    &self,
    env: napi_env,
    val: WeakBindingCell<rspack_core::EntryData>,
  ) -> napi::Result<napi::sys::napi_value> {
    let entry_data = EntryDataDTO {
      i: val,
      compiler_reference: None,
    };
    unsafe { ToNapiValue::to_napi_value(env, entry_data) }
  }

  fn allocate_entry_options(
    &self,
    env: napi_env,
    val: WeakBindingCell<rspack_core::EntryOptions>,
  ) -> napi::Result<napi::sys::napi_value> {
    let entry_options = EntryOptionsDTO { i: val };
    unsafe { ToNapiValue::to_napi_value(env, entry_options) }
  }
}

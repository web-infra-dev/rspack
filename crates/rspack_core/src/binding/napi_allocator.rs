use std::{cell::RefCell, sync::Arc};

use napi::sys::{napi_env, napi_value};
use once_cell::sync::OnceCell;

use crate::{AssetInfo, EntryData, EntryOptions};

thread_local! {
  static NAPI_ALLOCATOR: OnceCell<Box<dyn NapiAllocator>> = OnceCell::default();
}

pub trait NapiAllocator {
  fn allocate_asset_info(
    &self,
    env: napi_env,
    val: &Arc<RefCell<Box<AssetInfo>>>,
  ) -> napi::Result<napi_value>;
  fn allocate_entry_data(
    &self,
    env: napi_env,
    val: &Arc<RefCell<Box<EntryData>>>,
  ) -> napi::Result<napi_value>;
  fn allocate_entry_options(
    &self,
    env: napi_env,
    val: &Arc<RefCell<Box<EntryOptions>>>,
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

use napi_derive::napi;
use rspack_core::cache::persistent::storage::StorageOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawFileSystemOption {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
}

// If we have multiple `Storage`, change to `napi::Either`
pub type RawStorageOption = RawFileSystemOption;

impl From<RawStorageOption> for StorageOptions {
  fn from(_value: RawStorageOption) -> Self {
    StorageOptions::FileSystem
  }
}

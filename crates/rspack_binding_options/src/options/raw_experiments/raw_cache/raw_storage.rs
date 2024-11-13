use napi_derive::napi;
use rspack_core::cache::persistent::storage::StorageOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStorageOptions {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
}

impl From<RawStorageOptions> for StorageOptions {
  fn from(_value: RawStorageOptions) -> Self {
    StorageOptions::FileSystem
  }
}

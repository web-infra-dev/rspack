use napi_derive::napi;
use rspack_core::cache::persistent::storage::StorageOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStorageOptions {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
}

impl RawStorageOptions {
  pub(super) fn normalize(self) -> rspack_error::Result<StorageOptions> {
    match self.r#type.as_str() {
      "filesystem" => Ok(StorageOptions::FileSystem {
        directory: self.directory.into(),
      }),
      storage_type => Err(rspack_error::error!(
        "unsupported storage type {storage_type}"
      )),
    }
  }
}

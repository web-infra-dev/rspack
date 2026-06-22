use napi_derive::napi;
use rspack_core::cache::persistent::storage::StorageOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStorageOptions {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
  pub max_age: u32,
  pub max_generations: u32,
}

impl RawStorageOptions {
  pub(super) fn normalize(self) -> rspack_error::Result<(StorageOptions, u64, u32)> {
    match self.r#type.as_str() {
      "filesystem" => Ok((
        StorageOptions::FileSystem {
          directory: self.directory.into(),
        },
        self.max_age.into(),
        self.max_generations,
      )),
      storage_type => Err(rspack_error::error!(
        "unsupported storage type {storage_type}"
      )),
    }
  }
}

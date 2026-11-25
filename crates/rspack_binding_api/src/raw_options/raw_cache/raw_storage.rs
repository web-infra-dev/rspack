use napi_derive::napi;
use rspack_core::StorageOptions;
use rspack_fs::AbsPath;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStorageOptions {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
  pub location: String,
}

impl RawStorageOptions {
  pub(super) fn normalize(self) -> rspack_error::Result<StorageOptions> {
    match self.r#type.as_str() {
      "filesystem" => Ok(StorageOptions::FileSystem {
        directory: AbsPath::new(&self.directory)
          .map_err(|e| rspack_error::error!("invalid cache storage directory: {e}"))?,
        location: AbsPath::new(&self.location)
          .map_err(|e| rspack_error::error!("invalid cache storage location: {e}"))?,
      }),
      storage_type => Err(rspack_error::error!(
        "unsupported storage type {storage_type}"
      )),
    }
  }
}

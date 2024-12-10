use std::path::PathBuf;

use napi_derive::napi;
use rspack_core::cache::persistent::storage::{PackStorageOptions, StorageOptions};

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStorageOptions {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
}

impl From<RawStorageOptions> for StorageOptions {
  fn from(value: RawStorageOptions) -> Self {
    match value.r#type.as_str() {
      "filesystem" => {
        let root: PathBuf = value.directory.into();
        StorageOptions::FileSystem(PackStorageOptions {
          temp_root: root.join(".temp"),
          root,
          bucket_size: 1024,
          pack_size: 1024,
          expire: 7 * 24 * 60 * 60 * 1000,
        })
      }
      s => panic!("unsupport storage type {s}"),
    }
  }
}

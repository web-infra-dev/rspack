use std::num::NonZeroU32;

use napi_derive::napi;
use rspack_core::cache::persistent::storage::StorageOptions;
use rspack_error::error;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStorageOptions {
  #[napi(ts_type = r#""filesystem""#)]
  pub r#type: String,
  pub directory: String,
  pub max_versions: Option<u32>,
}

impl RawStorageOptions {
  pub(super) fn normalize(self) -> rspack_error::Result<(StorageOptions, Option<NonZeroU32>)> {
    match self.r#type.as_str() {
      "filesystem" => Ok((
        StorageOptions::FileSystem {
          directory: self.directory.into(),
        },
        self
          .max_versions
          .map(|value| {
            NonZeroU32::new(value)
              .ok_or_else(|| error!("cache.storage.maxVersions must be greater than 0"))
          })
          .transpose()?,
      )),
      storage_type => Err(error!("unsupported storage type {storage_type}")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_zero_max_versions() {
    let result = RawStorageOptions {
      r#type: "filesystem".to_string(),
      directory: "cache".to_string(),
      max_versions: Some(0),
    }
    .normalize();

    assert!(
      result
        .expect_err("zero maxVersions should be rejected")
        .to_string()
        .contains("cache.storage.maxVersions must be greater than 0")
    );
  }
}

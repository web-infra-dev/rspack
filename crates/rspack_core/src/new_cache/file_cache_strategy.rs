use rspack_error::Result;
use rspack_paths::Utf8PathBuf;

use super::{CacheData, Etag};

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
#[derive(Debug, Default)]
pub struct FileCacheStrategy;

impl FileCacheStrategy {
  pub async fn store(
    &self,
    _identifier: String,
    _etag: Option<Etag>,
    _data: CacheData,
  ) -> Result<()> {
    todo!("implement filesystem cache store")
  }

  pub async fn restore(&self, _identifier: &str, _etag: Option<&str>) -> Result<Option<CacheData>> {
    todo!("implement filesystem cache restore")
  }

  pub async fn store_build_dependencies(&self, _dependencies: Vec<Utf8PathBuf>) -> Result<()> {
    todo!("implement filesystem cache build dependencies store")
  }

  pub async fn after_all_stored(&self) -> Result<()> {
    todo!("implement filesystem cache finalization")
  }

  pub async fn clear(&self) -> Result<()> {
    todo!("implement filesystem cache clear")
  }
}

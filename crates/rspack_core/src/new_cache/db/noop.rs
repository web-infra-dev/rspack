use rspack_error::Result;

use crate::new_cache::{
  CacheKey,
  db::{Database, DatabaseFamily, DatabaseValue},
};

pub struct NoopDatabase;

impl NoopDatabase {
  #[cfg(target_family = "wasm")]
  pub fn open(
    _base_path: rspack_paths::Utf8PathBuf,
    _path: rspack_paths::Utf8PathBuf,
    _readonly: bool,
  ) -> Result<Self> {
    Ok(Self)
  }
}

impl Database for NoopDatabase {
  fn get(&self, _family: DatabaseFamily, _key: &CacheKey) -> Result<Option<DatabaseValue>> {
    Ok(None)
  }

  fn is_empty(&self) -> bool {
    true
  }

  fn write_batch(&self, _writes: Vec<(DatabaseFamily, CacheKey, Vec<u8>)>) -> Result<()> {
    Ok(())
  }

  fn compact(&self) -> Result<()> {
    Ok(())
  }

  fn reset(&mut self) -> Result<()> {
    Ok(())
  }

  fn cleanup_stale(&self) -> Result<()> {
    Ok(())
  }

  fn shutdown(&self) -> Result<()> {
    Ok(())
  }
}

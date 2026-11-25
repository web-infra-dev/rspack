use rayon::iter::ParallelIterator;
use rspack_error::Result;
use rspack_fs::AbsPath;

use crate::new_cache::{
  CacheKey,
  db::{DatabaseFamily, DatabaseValue},
};

pub struct NoopDatabase;

impl NoopDatabase {
  pub fn open(_base_path: AbsPath, _path: AbsPath, _readonly: bool) -> Result<Self> {
    Ok(Self)
  }

  pub fn get(&self, _family: DatabaseFamily, _key: &CacheKey) -> Result<Option<DatabaseValue>> {
    Ok(None)
  }

  pub fn is_empty(&self) -> bool {
    true
  }

  pub fn write_batch(
    &self,
    writes: impl ParallelIterator<Item = (DatabaseFamily, CacheKey, Vec<u8>)>,
  ) -> Result<usize> {
    Ok(writes.count())
  }

  pub fn compact(&self) -> Result<()> {
    Ok(())
  }

  pub fn has_unrecoverable_write_error(&self) -> bool {
    false
  }

  pub fn reset(&mut self) -> Result<()> {
    Ok(())
  }

  pub fn cleanup_stale(&self) -> Result<()> {
    Ok(())
  }

  pub fn shutdown(self) -> Result<()> {
    Ok(())
  }
}

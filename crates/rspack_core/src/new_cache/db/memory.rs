use std::sync::Arc;

use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap;

use super::{DatabaseFamily, DatabaseWrite};

pub type DatabaseValue = Arc<[u8]>;

#[derive(Debug, Default)]
pub struct Database {
  families: [FxHashMap<Vec<u8>, DatabaseValue>; DatabaseFamily::COUNT],
}

impl Database {
  pub fn open(_base_path: Utf8PathBuf, _path: Utf8PathBuf, _readonly: bool) -> Result<Self> {
    Ok(Self::default())
  }

  #[tracing::instrument(
    name = "new_cache:db_read",
    skip_all,
    level = "trace",
    target = "rspack_new_cache",
    fields(
      perfetto.track_name = "new_cache:db_read",
      perfetto.process_name = "Cache",
      family = ?family,
    )
  )]
  pub fn get(&self, family: DatabaseFamily, key: &[u8]) -> Result<Option<DatabaseValue>> {
    Ok(self.families[family.index()].get(key).cloned())
  }

  #[tracing::instrument(
    name = "new_cache:db_write",
    skip_all,
    level = "trace",
    target = "rspack_new_cache",
    fields(
      perfetto.track_name = "new_cache:db_write",
      perfetto.process_name = "Cache",
    )
  )]
  pub fn write_batch<'a>(
    &mut self,
    writes: impl IntoIterator<Item = DatabaseWrite<'a>>,
  ) -> Result<()> {
    for write in writes {
      self.families[write.family.index()].insert(write.key.to_vec(), Arc::from(write.value));
    }
    Ok(())
  }

  pub fn compact(&self) -> Result<()> {
    Ok(())
  }

  pub fn reset(&mut self) -> Result<()> {
    for family in &mut self.families {
      family.clear();
    }
    Ok(())
  }

  pub fn cleanup_stale(&self) {}

  pub fn shutdown(&mut self) -> Result<()> {
    self.reset()
  }
}

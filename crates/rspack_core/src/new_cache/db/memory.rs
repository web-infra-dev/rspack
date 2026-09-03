use std::sync::{Arc, Mutex, RwLock};

use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap;

use super::DatabaseFamily;
use crate::new_cache::CacheKey;

pub type DatabaseValue = Arc<[u8]>;

pub(crate) struct DatabaseBatch {
  writes: Mutex<[FxHashMap<CacheKey, DatabaseValue>; DatabaseFamily::COUNT]>,
}

impl DatabaseBatch {
  pub fn put(&self, family: DatabaseFamily, key: CacheKey, value: Vec<u8>) -> Result<()> {
    self
      .writes
      .lock()
      .expect("memory database batch mutex should not be poisoned")[family.index()]
    .insert(key, Arc::from(value));
    Ok(())
  }
}

#[derive(Debug)]
pub struct Database {
  families: RwLock<[FxHashMap<Vec<u8>, DatabaseValue>; DatabaseFamily::COUNT]>,
}

impl Database {
  pub fn open(_base_path: Utf8PathBuf, _path: Utf8PathBuf, _readonly: bool) -> Result<Self> {
    Ok(Self::noop())
  }

  pub fn noop() -> Self {
    Self {
      families: RwLock::new(Default::default()),
    }
  }

  pub fn get(&self, family: DatabaseFamily, key: &[u8]) -> Result<Option<DatabaseValue>> {
    Ok(
      self
        .families
        .read()
        .expect("memory database lock should not be poisoned")[family.index()]
      .get(key)
      .cloned(),
    )
  }

  pub fn is_empty(&self) -> bool {
    self
      .families
      .read()
      .expect("memory database lock should not be poisoned")
      .iter()
      .all(|family| family.is_empty())
  }

  pub fn write_batch(&self, write: impl FnOnce(&DatabaseBatch) -> Result<()>) -> Result<()> {
    let batch = DatabaseBatch {
      writes: Mutex::new(Default::default()),
    };
    write(&batch)?;
    let mut families = self
      .families
      .write()
      .expect("memory database lock should not be poisoned");
    for (family, writes) in families.iter_mut().zip(
      batch
        .writes
        .into_inner()
        .expect("memory database batch mutex should not be poisoned"),
    ) {
      family.extend(
        writes
          .into_iter()
          .map(|(key, value)| (Vec::from(key.as_bytes()), value)),
      );
    }
    Ok(())
  }

  pub fn compact(&self) -> Result<()> {
    Ok(())
  }

  pub fn reset(&mut self) -> Result<()> {
    self.clear();
    Ok(())
  }

  fn clear(&self) {
    for family in self
      .families
      .write()
      .expect("memory database lock should not be poisoned")
      .iter_mut()
    {
      family.clear();
    }
  }

  pub fn cleanup_stale(&self) -> Result<()> {
    Ok(())
  }

  pub fn shutdown(&self) -> Result<()> {
    self.clear();
    Ok(())
  }
}

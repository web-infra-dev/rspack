use std::sync::{Arc, Mutex};

use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap;

use super::DatabaseFamily;

pub type DatabaseValue = Arc<[u8]>;

pub(crate) struct DatabaseBatch {
  writes: Mutex<[FxHashMap<Vec<u8>, DatabaseValue>; DatabaseFamily::COUNT]>,
}

impl DatabaseBatch {
  pub fn put(&self, family: DatabaseFamily, key: &[u8], value: Vec<u8>) -> Result<()> {
    self
      .writes
      .lock()
      .expect("memory database batch mutex should not be poisoned")[family.index()]
    .insert(key.to_vec(), Arc::from(value));
    Ok(())
  }
}

#[derive(Debug, Default)]
pub struct Database {
  families: [FxHashMap<Vec<u8>, DatabaseValue>; DatabaseFamily::COUNT],
}

impl Database {
  pub fn open(_base_path: Utf8PathBuf, _path: Utf8PathBuf, _readonly: bool) -> Result<Self> {
    Ok(Self::default())
  }

  pub fn get(&self, family: DatabaseFamily, key: &[u8]) -> Result<Option<DatabaseValue>> {
    Ok(self.families[family.index()].get(key).cloned())
  }

  pub fn write_batch(&mut self, write: impl FnOnce(&DatabaseBatch) -> Result<()>) -> Result<()> {
    let batch = DatabaseBatch {
      writes: Mutex::new(Default::default()),
    };
    write(&batch)?;
    for (family, writes) in self.families.iter_mut().zip(
      batch
        .writes
        .into_inner()
        .expect("memory database batch mutex should not be poisoned"),
    ) {
      family.extend(writes);
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

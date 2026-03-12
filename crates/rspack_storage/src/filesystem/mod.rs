mod db;
mod options;
mod scope_fs;

use std::sync::Mutex;

use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::Receiver;

pub use self::options::FileSystemOptions;
use self::{db::DB, scope_fs::ScopeFileSystem};
use crate::{Key, Result, Storage, Value};

/// File system-based persistent storage implementation
///
/// Uses pack file format to merge multiple small files for better I/O efficiency.
/// Maintains an in-memory buffer for batched writes to reduce disk operations.
#[derive(Debug)]
pub struct FileSystemStorage {
  /// Underlying database responsible for pack file read/write
  db: DB,
  /// In-memory staged update operations, grouped by scope
  /// Value of Some(value) indicates write, None indicates deletion
  updates: Mutex<HashMap<String, HashMap<Vec<u8>, Option<Vec<u8>>>>>,
}

impl FileSystemStorage {
  /// Creates a new file system storage instance
  pub fn new(options: FileSystemOptions) -> Self {
    let fs = ScopeFileSystem::new(options.directory, options.fs);
    Self {
      db: DB::new(fs, options.max_pack_size),
      updates: Default::default(),
    }
  }
}

#[async_trait::async_trait]
impl Storage for FileSystemStorage {
  async fn load(&self, scope: &'static str) -> Result<Vec<(Key, Value)>> {
    let data = self.db.load(scope).await?;
    Ok(data)
  }

  fn set(&self, scope: &'static str, key: Vec<u8>, value: Vec<u8>) {
    let mut updates = self.updates.lock().expect("should get lock");
    let scope_update = updates.entry(scope.to_string()).or_default();
    scope_update.insert(key, Some(value));
  }

  fn remove(&self, scope: &'static str, key: &[u8]) {
    let mut updates = self.updates.lock().expect("should get lock");
    let scope_update = updates.entry(scope.to_string()).or_default();
    scope_update.insert(key.to_vec(), None);
  }

  fn trigger_save(&self) -> Result<Receiver<Result<()>>> {
    // Take all pending updates and clear the memory buffer
    let updates = std::mem::take(&mut *self.updates.lock().expect("should get lock"));
    // Convert updates to Vec format and trigger database save
    let res = self.db.save(
      updates
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect(),
    )?;
    Ok(res)
  }

  async fn reset(&self) {
    let _ = self.db.reset().await;
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    let names = self.db.bucket_names().await?;
    Ok(names)
  }
}

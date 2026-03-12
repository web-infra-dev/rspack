mod db;
mod options;
mod scope_fs;

use std::sync::Mutex;

use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::Receiver;

pub use self::options::FileSystemOptions;
use self::{db::DB, scope_fs::ScopeFileSystem};
use crate::{Key, Result, Storage, Value};

#[derive(Debug)]
pub struct FileSystemStorage {
  db: DB,
  updates: Mutex<HashMap<String, HashMap<Vec<u8>, Option<Vec<u8>>>>>,
}

impl FileSystemStorage {
  pub fn new(options: FileSystemOptions) -> Self {
    // TODO add clean feature
    // TODO clean load failed buckets?
    let fs = ScopeFileSystem::new(options.directory, options.fs);
    Self {
      db: DB::new(options.max_pack_size, fs),
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
    let updates = std::mem::take(&mut *self.updates.lock().expect("should get lock"));
    let t = self.db.save(
      updates
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect(),
    )?;
    Ok(t)
  }

  async fn reset(&self) {
    let _ = self.db.reset().await;
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    let names = self.db.bucket_names().await?;
    Ok(names)
  }
}

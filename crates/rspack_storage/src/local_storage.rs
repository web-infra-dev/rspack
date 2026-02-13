use std::sync::{Arc, Mutex};

use rspack_paths::AssertUtf8;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::Receiver;

use super::{
  PackStorageOptions, Result, Storage,
  db::{DB, Options},
};

#[derive(Debug)]
pub struct LocalStorage {
  db: DB,
  updates: Mutex<HashMap<String, HashMap<Vec<u8>, Option<Vec<u8>>>>>,
}

impl LocalStorage {
  pub fn new(options: PackStorageOptions) -> Self {
    // TODO add clean feature
    let root = options.root.join(options.version.clone());
    Self {
      db: DB::new(root.assert_utf8(), Options::default(), options.fs.0.clone()),
      updates: Default::default(),
    }
  }
}

#[async_trait::async_trait]
impl Storage for LocalStorage {
  async fn init(&self) -> Result<()> {
    self.db.init().await?;
    Ok(())
  }
  async fn load(&self, scope: &'static str) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>> {
    let data = self.db.load(scope).await?;
    Ok(
      data
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect(),
    )
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

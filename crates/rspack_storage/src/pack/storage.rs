use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use rspack_error::Result;
use rspack_paths::AssertUtf8;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::Receiver;

use super::{PackFs, PackOptions, ScopeManager, ScopeUpdate, SplitPackStrategy};
use crate::{Storage, StorageContent, StorageItemKey, StorageItemValue};

pub type ScopeUpdates = HashMap<&'static str, ScopeUpdate>;
#[derive(Debug)]
pub struct PackStorage {
  manager: ScopeManager,
  updates: Mutex<ScopeUpdates>,
}

pub struct PackStorageOptions {
  root: PathBuf,
  temp_root: PathBuf,
  fs: Arc<dyn PackFs>,
  bucket_size: usize,
  pack_size: usize,
  expire: u64,
}

impl PackStorage {
  pub fn new(options: PackStorageOptions) -> Self {
    Self {
      manager: ScopeManager::new(
        Arc::new(PackOptions {
          bucket_size: options.bucket_size,
          pack_size: options.pack_size,
          expire: options.expire,
        }),
        Arc::new(SplitPackStrategy::new(
          options.root.assert_utf8(),
          options.temp_root.assert_utf8(),
          options.fs,
        )),
      ),
      updates: Default::default(),
    }
  }
}

#[async_trait::async_trait]
impl Storage for PackStorage {
  async fn get_all(&self, name: &'static str) -> Result<StorageContent> {
    self.manager.get_all(name).await
  }
  fn set(&self, scope: &'static str, key: StorageItemKey, value: StorageItemValue) {
    let mut updates = self.updates.lock().expect("should get lock");
    let scope_update = updates.entry(scope).or_default();
    scope_update.insert(key, Some(value));
  }
  fn remove(&self, scope: &'static str, key: &StorageItemKey) {
    let mut updates = self.updates.lock().expect("should get lock");
    let scope_update = updates.entry(scope).or_default();
    scope_update.insert(key.to_vec(), None);
  }
  fn idle(&self) -> Result<Receiver<Result<()>>> {
    self.manager.save(std::mem::take(
      &mut *self.updates.lock().expect("should get lock"),
    ))
  }
}

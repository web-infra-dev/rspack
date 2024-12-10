mod data;
mod fs;
mod manager;
mod strategy;

use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use data::PackOptions;
pub use fs::{PackBridgeFS, PackFS};
use manager::ScopeManager;
use rspack_error::Result;
use rspack_paths::AssertUtf8;
use rustc_hash::FxHashMap as HashMap;
use strategy::{ScopeUpdate, SplitPackStrategy};
use tokio::sync::oneshot::Receiver;

use crate::{Storage, StorageContent, StorageItemKey, StorageItemValue};

pub type ScopeUpdates = HashMap<&'static str, ScopeUpdate>;
#[derive(Debug)]
pub struct PackStorage {
  manager: ScopeManager,
  updates: Mutex<ScopeUpdates>,
}

#[derive(Debug, Clone)]
pub struct PackStorageOptions {
  pub root: PathBuf,
  pub temp_root: PathBuf,
  pub bucket_size: usize,
  pub pack_size: usize,
  pub expire: u64,
}

impl PackStorage {
  pub fn new(options: PackStorageOptions, fs: Arc<dyn PackFS>) -> Self {
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
          fs,
        )),
      ),
      updates: Default::default(),
    }
  }
}

#[async_trait::async_trait]
impl Storage for PackStorage {
  async fn load(&self, name: &'static str) -> Result<StorageContent> {
    let res = self.manager.load(name).await;
    println!("get all {name:?} {res:?}");
    res
  }
  fn set(&self, scope: &'static str, key: StorageItemKey, value: StorageItemValue) {
    let mut updates = self.updates.lock().expect("should get lock");
    let scope_update = updates.entry(scope).or_default();
    scope_update.insert(key, Some(value));
  }
  fn remove(&self, scope: &'static str, key: &[u8]) {
    let mut updates = self.updates.lock().expect("should get lock");
    let scope_update = updates.entry(scope).or_default();
    scope_update.insert(key.to_vec(), None);
  }
  fn trigger_save(&self) -> Result<Receiver<Result<()>>> {
    self.manager.save(std::mem::take(
      &mut *self.updates.lock().expect("should get lock"),
    ))
  }
}

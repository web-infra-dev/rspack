mod data;
mod manager;
mod strategy;

use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use data::{PackOptions, RootOptions};
use manager::ScopeManager;
use rspack_paths::AssertUtf8;
use rustc_hash::FxHashMap as HashMap;
use strategy::{ScopeUpdate, SplitPackStrategy};
use tokio::sync::oneshot::Receiver;

use crate::{FileSystem, ItemKey, ItemPairs, ItemValue, Storage, error::Result};

pub type ScopeUpdates = HashMap<&'static str, ScopeUpdate>;
#[derive(Debug)]
pub struct PackStorage {
  pub manager: ScopeManager,
  pub updates: Mutex<ScopeUpdates>,
}

pub struct PackStorageOptions {
  pub root: PathBuf,
  pub temp_root: PathBuf,
  pub fs: Arc<dyn FileSystem>,
  pub bucket_size: usize,
  pub pack_size: usize,
  pub expire: u64,
  pub version: String,
  pub clean: bool,
  pub fresh_generation: Option<usize>,
  pub release_generation: Option<usize>,
}

impl PackStorage {
  pub fn new(options: PackStorageOptions) -> Self {
    Self {
      manager: ScopeManager::new(
        Arc::new(RootOptions {
          root: options.root.clone().assert_utf8(),
          expire: options.expire,
          clean: options.clean,
        }),
        Arc::new(PackOptions {
          bucket_size: options.bucket_size,
          pack_size: options.pack_size,
        }),
        Arc::new(SplitPackStrategy::new(
          options.root.join(&options.version).assert_utf8(),
          options.temp_root.join(&options.version).assert_utf8(),
          options.fs,
          options.fresh_generation,
          options.release_generation,
        )),
      ),
      updates: Default::default(),
    }
  }
}

#[async_trait::async_trait]
impl Storage for PackStorage {
  async fn load(&self, name: &'static str) -> Result<ItemPairs> {
    self.manager.load(name).await
  }
  fn set(&self, scope: &'static str, key: ItemKey, value: ItemValue) {
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
  async fn reset(&self) {
    self.manager.reset().await;
  }
  async fn scopes(&self) -> Result<Vec<String>> {
    self.manager.scopes().await
  }
}

mod db;
mod maintenance;
mod meta;
mod options;
mod retention;
mod scope_fs;

use rustc_hash::FxHashMap as HashMap;

use self::{db::DB, maintenance::Maintenance, meta::Meta, scope_fs::ScopeFileSystem};
pub use self::{options::FileSystemOptions, retention::VersionRetention};
use crate::{Result, Storage};

/// Type alias for in-memory update changes: key -> optional_value
type BucketChangesMap = HashMap<Vec<u8>, Option<Vec<u8>>>;

/// File system-based persistent storage implementation
#[derive(Debug)]
pub struct FileSystemStorage {
  /// Underlying database responsible for pack file read/write
  db: DB,
  /// In-memory staged update operations, grouped by scope
  /// Value of Some(value) indicates write, None indicates deletion
  updates: HashMap<String, BucketChangesMap>,
  /// Storage options
  options: FileSystemOptions,
  /// Version metadata refresh and cleanup, run after a successful DB save
  maintenance: Maintenance,
}

impl FileSystemStorage {
  /// Creates a new file system storage instance
  pub fn new(options: FileSystemOptions) -> Self {
    let fs = ScopeFileSystem::new(options.directory.clone(), options.fs.clone());

    Self {
      db: DB::new(fs.child_fs(&options.version)),
      updates: Default::default(),
      maintenance: Maintenance::new(
        fs,
        options.version.clone(),
        options.expire,
        options.retention.clone(),
      ),
      options,
    }
  }
}

#[async_trait::async_trait]
impl Storage for FileSystemStorage {
  async fn load(&self, scope: &'static str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let data = self.db.load(scope).await?;
    Ok(data)
  }

  fn set(&mut self, scope: &'static str, key: Vec<u8>, value: Vec<u8>) {
    let scope_update = self.updates.entry(scope.to_string()).or_default();
    scope_update.insert(key, Some(value));
  }

  fn remove(&mut self, scope: &'static str, key: &[u8]) {
    let scope_update = self.updates.entry(scope.to_string()).or_default();
    scope_update.insert(key.to_vec(), None);
  }

  fn save(&mut self) {
    // Take all pending updates and clear the memory buffer
    let updates = std::mem::take(&mut self.updates);

    // Enqueue the write to the background task queue; errors are reported internally.
    // Call flush() to wait until the write has fully completed.
    let before_save = self.maintenance.clone();
    let on_success = self.maintenance.clone();
    let on_failure = self.maintenance.clone();
    self.db.save(
      updates
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect(),
      self.options.max_pack_size,
      async move { before_save.prepare().await },
      async move { on_success.run().await },
      async move { on_failure.cancel().await },
    );
  }

  fn reset(&mut self, scope: &'static str) {
    // Discard any pending writes for this scope so they don't race with the reset
    self.updates.remove(scope);
    // Enqueue the directory deletion immediately into the task queue
    self.db.reset(scope);
  }

  async fn flush(&self) {
    self.db.flush().await;
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    let names = self.db.bucket_names().await?;
    Ok(names)
  }
}

#[cfg(test)]
mod tests {
  use std::{num::NonZeroUsize, sync::Arc};

  use futures::future::join_all;
  use rspack_fs::MemoryFileSystem;

  use super::{FileSystemOptions, FileSystemStorage, ScopeFileSystem, VersionRetention};
  use crate::Storage;

  async fn save_version(fs: Arc<MemoryFileSystem>, version: &str, retention_scope: &str) {
    let mut storage = FileSystemStorage::new(FileSystemOptions {
      directory: "/cache".into(),
      version: version.into(),
      max_pack_size: 500 * 1024,
      expire: 0,
      retention: Some(VersionRetention::new(
        retention_scope.into(),
        NonZeroUsize::new(2).expect("non-zero retention limit"),
      )),
      fs,
    });
    storage.set("scope", b"key".to_vec(), b"value".to_vec());
    storage.save();
    storage.flush().await;
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_remove_old_versions_for_the_same_compiler() {
    let fs = Arc::new(MemoryFileSystem::default());

    save_version(fs.clone(), "a-v1", "compiler-a").await;
    save_version(fs.clone(), "b-v1", "compiler-b").await;
    save_version(fs.clone(), "a-v2", "compiler-a").await;
    save_version(fs.clone(), "a-v3", "compiler-a").await;

    let root = ScopeFileSystem::new("/cache".into(), fs);
    let mut versions = root.list_child().await.expect("cache root should exist");
    versions.retain(|version| !version.starts_with(['_', '.']));
    versions.sort();
    assert_eq!(versions, vec!["a-v2", "a-v3", "b-v1"]);
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  #[cfg_attr(miri, ignore)]
  async fn should_preserve_metadata_from_concurrent_compilers() {
    let fs = Arc::new(MemoryFileSystem::default());

    join_all((0..16).map(|index| {
      let fs = fs.clone();
      async move {
        save_version(
          fs,
          &format!("compiler-{index}-v1"),
          &format!("compiler-{index}"),
        )
        .await;
      }
    }))
    .await;
    for index in 0..16 {
      save_version(
        fs.clone(),
        &format!("compiler-{index}-v2"),
        &format!("compiler-{index}"),
      )
      .await;
      save_version(
        fs.clone(),
        &format!("compiler-{index}-v3"),
        &format!("compiler-{index}"),
      )
      .await;
    }

    let root = ScopeFileSystem::new("/cache".into(), fs);
    let versions = root.list_child().await.expect("cache root should exist");
    assert_eq!(
      versions
        .iter()
        .filter(|version| !version.starts_with(['_', '.']))
        .count(),
      16 * 2
    );
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_not_create_retention_metadata_when_disabled() {
    let fs = Arc::new(MemoryFileSystem::default());
    let mut storage = FileSystemStorage::new(FileSystemOptions {
      directory: "/cache".into(),
      version: "v1".into(),
      max_pack_size: 500 * 1024,
      expire: 0,
      retention: None,
      fs: fs.clone(),
    });
    storage.set("scope", b"key".to_vec(), b"value".to_vec());
    storage.save();
    storage.flush().await;

    let root = ScopeFileSystem::new("/cache".into(), fs);
    let entries = root.list_child().await.expect("cache root should exist");
    assert!(!entries.iter().any(|entry| entry == ".retention"));
    assert!(!entries.iter().any(|entry| entry == ".maintenance.lock"));
  }
}

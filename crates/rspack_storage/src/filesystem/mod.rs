mod db;
mod meta;
mod options;
mod scope_fs;

use std::{
  num::NonZeroU32,
  sync::{Arc, Mutex},
};

use rustc_hash::FxHashMap as HashMap;

pub use self::options::FileSystemOptions;
use self::{db::DB, meta::Meta, scope_fs::ScopeFileSystem};
use crate::{Result, Storage};

/// Type alias for in-memory update changes: key -> optional_value
type BucketChangesMap = HashMap<Vec<u8>, Option<Vec<u8>>>;

async fn refresh_metadata(
  fs: ScopeFileSystem,
  version: String,
  expire: u64,
  max_versions: Option<NonZeroU32>,
  next_meta_refresh_time: Arc<Mutex<u64>>,
) {
  let now = Meta::current_timestamp();
  if *next_meta_refresh_time.lock().expect("should get lock") > now {
    return;
  }

  let mut meta = match Meta::load(&fs).await {
    Ok(meta) => meta,
    Err(error) if error.is_not_found() => Meta::default(),
    Err(_) => return,
  };
  let versions = if max_versions.is_some() {
    fs.list_child().await.unwrap_or_default()
  } else {
    Vec::new()
  };
  let Ok((removed_versions, next_refresh_time)) = meta
    .refresh(&version, expire, max_versions, &versions)
    .await
  else {
    return;
  };
  if meta.save(&fs).await.is_err() {
    return;
  }

  for version in removed_versions {
    let _ = fs.child_fs(&version).remove().await;
  }
  *next_meta_refresh_time.lock().expect("should get lock") = next_refresh_time;
}

/// File system-based persistent storage implementation
#[derive(Debug)]
pub struct FileSystemStorage {
  /// Root filesystem for metadata operations
  fs: ScopeFileSystem,
  /// Underlying database responsible for pack file read/write
  db: DB,
  /// In-memory staged update operations, grouped by scope
  /// Value of Some(value) indicates write, None indicates deletion
  updates: HashMap<String, BucketChangesMap>,
  /// Storage options
  options: FileSystemOptions,
  /// Next scheduled time for metadata refresh (cleanup + access time update)
  next_meta_refresh_time: Arc<Mutex<u64>>,
}

impl FileSystemStorage {
  /// Creates a new file system storage instance
  pub fn new(options: FileSystemOptions) -> Self {
    let fs = ScopeFileSystem::new(options.directory.clone(), options.fs.clone());

    Self {
      db: DB::new(fs.child_fs(&options.version)),
      updates: Default::default(),
      next_meta_refresh_time: Default::default(),
      fs,
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
    self.db.save(
      updates
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect(),
      self.options.max_pack_size,
    );

    if self.options.max_versions.is_none() {
      tokio::spawn(refresh_metadata(
        self.fs.clone(),
        self.options.version.clone(),
        self.options.expire,
        None,
        self.next_meta_refresh_time.clone(),
      ));
    }
  }

  fn reset(&mut self, scope: &'static str) {
    // Discard any pending writes for this scope so they don't race with the reset
    self.updates.remove(scope);
    // Enqueue the directory deletion immediately into the task queue
    self.db.reset(scope);
  }

  async fn flush(&self) {
    self.db.flush().await;
    if self.options.max_versions.is_some() && !self.db.is_readonly() {
      refresh_metadata(
        self.fs.clone(),
        self.options.version.clone(),
        self.options.expire,
        self.options.max_versions,
        self.next_meta_refresh_time.clone(),
      )
      .await;
    }
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    let names = self.db.bucket_names().await?;
    Ok(names)
  }
}

#[cfg(test)]
mod tests {
  use std::{num::NonZeroU32, sync::Arc};

  use futures::future::join_all;
  use rspack_fs::MemoryFileSystem;

  use super::{FileSystemOptions, FileSystemStorage, ScopeFileSystem};
  use crate::Storage;

  async fn save_version(fs: Arc<MemoryFileSystem>, version: &str) {
    let mut storage = FileSystemStorage::new(FileSystemOptions {
      directory: "/cache".into(),
      version: version.into(),
      max_pack_size: 500 * 1024,
      expire: 0,
      max_versions: NonZeroU32::new(2),
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

    save_version(fs.clone(), "a-v1").await;
    save_version(fs.clone(), "b-v1").await;
    save_version(fs.clone(), "a-v2").await;
    save_version(fs.clone(), "a-v3").await;

    let root = ScopeFileSystem::new("/cache".into(), fs);
    let mut versions = root.list_child().await.expect("cache root should exist");
    versions.retain(|version| !version.starts_with(['_', '.']));
    versions.sort();
    assert_eq!(versions, vec!["a-v2", "a-v3", "b-v1"]);
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  #[cfg_attr(miri, ignore)]
  async fn should_recover_from_concurrent_metadata_updates() {
    let fs = Arc::new(MemoryFileSystem::default());

    join_all((0..8).map(|index| {
      let fs = fs.clone();
      async move { save_version(fs, &format!("scope{index}-v1")).await }
    }))
    .await;

    for index in 0..8 {
      save_version(fs.clone(), &format!("scope{index}-v2")).await;
      save_version(fs.clone(), &format!("scope{index}-v3")).await;
    }

    let root = ScopeFileSystem::new("/cache".into(), fs);
    let versions = root.list_child().await.expect("cache root should exist");
    assert_eq!(
      versions
        .iter()
        .filter(|version| !version.starts_with(['_', '.']))
        .count(),
      8 * 2
    );
  }
}

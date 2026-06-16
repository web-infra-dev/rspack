mod db;
mod meta;
mod options;
mod scope_fs;
mod task_queue;

use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap as HashMap;

pub use self::options::FileSystemOptions;
use self::{db::DB, meta::Meta, scope_fs::ScopeFileSystem, task_queue::TaskQueue};
use crate::{Result, Storage};

/// Type alias for in-memory update changes: key -> optional_value
type BucketChangesMap = HashMap<Vec<u8>, Option<Vec<u8>>>;

async fn refresh_metadata(
  fs: ScopeFileSystem,
  version: String,
  expire: u64,
  max_generations: Option<u32>,
  next_meta_refresh_time: Arc<Mutex<u64>>,
) {
  let now = Meta::current_timestamp();
  if *next_meta_refresh_time.lock().expect("should get lock") > now {
    return;
  }

  // Missing metadata is normal for a newly-created cache scope.
  let mut meta = match Meta::load(&fs).await {
    Ok(meta) => meta,
    Err(error) if error.is_not_found() => Meta::default(),
    Err(_) => return,
  };
  // Generation cleanup needs the current compiler scope's version directories.
  let versions = if max_generations.is_some() {
    fs.list_child().await.unwrap_or_default()
  } else {
    Vec::new()
  };
  let Ok((removed_versions, next_refresh_time)) = meta
    .refresh(&version, expire, max_generations, &versions)
    .await
  else {
    return;
  };
  if meta.save(&fs).await.is_err() {
    return;
  }

  // Persist metadata before deleting directories so concurrent refreshes can
  // recover even if removal is interrupted.
  for version in removed_versions {
    let _ = fs.child_fs(&version).remove().await;
  }
  *next_meta_refresh_time.lock().expect("should get lock") = next_refresh_time;
}

/// File system-based persistent storage implementation
#[derive(Debug)]
pub struct FileSystemStorage {
  /// Compiler-scoped filesystem for metadata operations
  fs: ScopeFileSystem,
  /// Underlying database responsible for pack file read/write
  db: DB,
  /// Sequential queue for filesystem writes and follow-up maintenance
  task_queue: TaskQueue,
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
    // All metadata and DB operations are scoped to the current compiler, so
    // cleanup never touches cache entries owned by another compiler.
    let fs = ScopeFileSystem::new(options.directory.clone(), options.fs.clone())
      .child_fs(&options.compiler_scope);

    Self {
      db: DB::new(fs.child_fs(&options.version)),
      task_queue: TaskQueue::default(),
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

    // Queue the write and metadata refresh together so cleanup observes the
    // latest version without blocking `save()`.
    let db = self.db.clone();
    let changes = updates
      .into_iter()
      .map(|(k, v)| (k, v.into_iter().collect()))
      .collect();
    let max_pack_size = self.options.max_pack_size;
    let fs = self.fs.clone();
    let version = self.options.version.clone();
    let expire = self.options.expire;
    let max_generations = self.options.max_generations;
    let next_meta_refresh_time = self.next_meta_refresh_time.clone();

    self.task_queue.add_task(async move {
      if db.save(changes, max_pack_size).await {
        refresh_metadata(fs, version, expire, max_generations, next_meta_refresh_time).await;
      }
    });
  }

  fn reset(&mut self, scope: &'static str) {
    // Discard any pending writes for this scope so they don't race with the reset
    self.updates.remove(scope);
    // Queue the directory deletion so it is sequenced with saves.
    let db = self.db.clone();
    self.task_queue.add_task(async move {
      db.reset(scope).await;
    });
  }

  async fn flush(&self) {
    self.task_queue.flush().await;
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    let names = self.db.bucket_names().await?;
    Ok(names)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use futures::future::join_all;
  use rspack_fs::MemoryFileSystem;

  use super::{FileSystemOptions, FileSystemStorage, ScopeFileSystem};
  use crate::Storage;

  async fn save_version(fs: Arc<MemoryFileSystem>, compiler_scope: &str, version: &str) {
    let mut storage = FileSystemStorage::new(FileSystemOptions {
      directory: "/cache".into(),
      compiler_scope: compiler_scope.into(),
      version: version.into(),
      max_pack_size: 500 * 1024,
      expire: 0,
      max_generations: Some(2),
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

    save_version(fs.clone(), "a", "v1").await;
    save_version(fs.clone(), "b", "v1").await;
    save_version(fs.clone(), "a", "v2").await;
    save_version(fs.clone(), "a", "v3").await;

    let root = ScopeFileSystem::new("/cache".into(), fs);
    let mut a_versions = root
      .child_fs("a")
      .list_child()
      .await
      .expect("compiler cache scope should exist");
    a_versions.retain(|version| !version.starts_with(['_', '.']));
    a_versions.sort();
    assert_eq!(a_versions, vec!["v2", "v3"]);

    let mut b_versions = root
      .child_fs("b")
      .list_child()
      .await
      .expect("compiler cache scope should exist");
    b_versions.retain(|version| !version.starts_with(['_', '.']));
    assert_eq!(b_versions, vec!["v1"]);
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  #[cfg_attr(miri, ignore)]
  async fn should_recover_from_concurrent_metadata_updates() {
    let fs = Arc::new(MemoryFileSystem::default());

    join_all((0..8).map(|index| {
      let fs = fs.clone();
      async move { save_version(fs, &format!("scope{index}"), "v1").await }
    }))
    .await;

    for index in 0..8 {
      save_version(fs.clone(), &format!("scope{index}"), "v2").await;
      save_version(fs.clone(), &format!("scope{index}"), "v3").await;
    }

    let root = ScopeFileSystem::new("/cache".into(), fs);
    for index in 0..8 {
      let scope = format!("scope{index}");
      let versions = root
        .child_fs(&scope)
        .list_child()
        .await
        .expect("compiler cache scope should exist");
      assert_eq!(
        versions
          .iter()
          .filter(|version| !version.starts_with(['_', '.']))
          .count(),
        2
      );
    }
  }
}

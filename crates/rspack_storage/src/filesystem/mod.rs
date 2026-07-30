mod cache_directory;
mod db;
mod meta;
mod options;
mod scope_fs;
mod task_queue;

use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap as HashMap;

pub use self::{cache_directory::CacheDirectory, options::FileSystemOptions};
use self::{db::DB, meta::Meta, scope_fs::ScopeFileSystem, task_queue::TaskQueue};
use crate::{Result, Storage};

/// Type alias for in-memory update changes: key -> optional_value
type BucketChangesMap = HashMap<Vec<u8>, Option<Vec<u8>>>;

const STALE_DIR_NAME: &str = "_stale";

async fn cleanup_stale_directories(stale_fs: ScopeFileSystem) -> Result<()> {
  stale_fs.ensure_exist().await?;

  let stale_directories = stale_fs
    .list_child()
    .await
    .unwrap_or_default()
    .into_iter()
    .filter(|child| CacheDirectory::parse(child).is_some())
    .map(|child| stale_fs.child_fs(child));

  for stale_directory in stale_directories {
    let _ = stale_directory.remove().await;
  }

  Ok(())
}

fn spawn_cleanup_stale_directories(stale_fs: ScopeFileSystem) {
  tokio::spawn(async move { cleanup_stale_directories(stale_fs).await });
}

async fn move_stale_directories(
  fs: &ScopeFileSystem,
  stale_fs: &ScopeFileSystem,
  stale_directories: Vec<CacheDirectory>,
) -> Result<()> {
  stale_fs.ensure_exist().await?;

  for directory in stale_directories {
    ScopeFileSystem::move_to(fs, stale_fs, directory.as_str()).await?;
  }
  Ok(())
}

async fn refresh_metadata(
  fs: ScopeFileSystem,
  stale_fs: ScopeFileSystem,
  cache_directory: CacheDirectory,
  expire: u64,
  next_meta_refresh_time: Arc<Mutex<u64>>,
) {
  let now = Meta::current_timestamp();
  if *next_meta_refresh_time.lock().expect("should get lock") > now {
    return;
  }

  // Missing metadata is normal for a newly-created cache directory.
  let mut meta = match Meta::load(&fs).await {
    Ok(meta) => meta,
    Err(error) if error.is_not_found() => Meta::default(),
    Err(_) => return,
  };
  let Ok((stale_directories, next_refresh_time)) = meta.refresh(&cache_directory, expire) else {
    return;
  };
  if meta.save(&fs).await.is_err() {
    return;
  }

  // Persist metadata before renaming directories so concurrent refreshes can
  // recover even if stale cleanup is interrupted.
  if move_stale_directories(&fs, &stale_fs, stale_directories)
    .await
    .is_err()
  {
    return;
  }
  spawn_cleanup_stale_directories(stale_fs);
  *next_meta_refresh_time.lock().expect("should get lock") = next_refresh_time;
}

/// File system-based persistent storage implementation
#[derive(Debug)]
pub struct FileSystemStorage {
  /// Filesystem for metadata operations
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
    let fs = ScopeFileSystem::new(options.directory.clone(), options.fs.clone());

    Self {
      db: DB::new(fs.child_fs(options.cache_directory.as_str())),
      task_queue: TaskQueue::default(),
      updates: Default::default(),
      next_meta_refresh_time: Default::default(),
      fs,
      options,
    }
  }

  pub fn stale_fs(&self) -> ScopeFileSystem {
    self.fs.child_fs(STALE_DIR_NAME)
  }
}

#[async_trait::async_trait]
impl Storage for FileSystemStorage {
  fn cleanup_stale(&self) {
    spawn_cleanup_stale_directories(self.stale_fs());
  }

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
    // latest compiler cache without blocking `save()`.
    let db = self.db.clone();
    let changes = updates
      .into_iter()
      .map(|(k, v)| (k, v.into_iter().collect()))
      .collect();
    let max_pack_size = self.options.max_pack_size;
    let fs = self.fs.clone();
    let stale_fs = self.stale_fs();
    let cache_directory = self.options.cache_directory.clone();
    let expire = self.options.expire;
    let next_meta_refresh_time = self.next_meta_refresh_time.clone();

    self.task_queue.add_task(async move {
      if db.save(changes, max_pack_size).await {
        refresh_metadata(
          fs,
          stale_fs,
          cache_directory,
          expire,
          next_meta_refresh_time,
        )
        .await;
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
  use super::{Result, ScopeFileSystem, cleanup_stale_directories};

  #[tokio::test]
  async fn removes_stale_cache_directories() -> Result<()> {
    let stale_fs = ScopeFileSystem::new_memory_fs("/cache/_stale".into());
    let stale = "rspack_v_0000000000000001";
    let ordinary = "ordinary-directory";

    stale_fs.ensure_exist().await?;
    stale_fs.child_fs(stale).ensure_exist().await?;
    stale_fs.child_fs(ordinary).ensure_exist().await?;

    cleanup_stale_directories(stale_fs.clone()).await?;

    let stale_children = stale_fs.list_child().await?;
    assert!(!stale_children.iter().any(|child| child == stale));
    assert!(stale_children.iter().any(|child| child == ordinary));

    Ok(())
  }
}

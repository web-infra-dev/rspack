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
  max_generations: u32,
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
  // Cleanup needs the current storage directory entries to keep metadata in
  // sync with versions that still exist on disk.
  let versions = fs.list_child().await.unwrap_or_default();
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

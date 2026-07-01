mod db;
mod meta;
mod options;
mod scope_fs;
mod task_queue;
mod version;

use std::sync::{Arc, Mutex};

use self::{db::DB, meta::Meta, scope_fs::ScopeFileSystem, task_queue::TaskQueue};
pub use self::{options::FileSystemOptions, version::Version};
use crate::{Error, Result, Storage, StorageUpdateTask, StorageUpdates};

#[derive(Default)]
struct UpdateTasks(Vec<StorageUpdateTask>);

impl std::fmt::Debug for UpdateTasks {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("UpdateTasks").field(&self.0.len()).finish()
  }
}

fn spawn_cleanup_stale_versions(stale_fs: ScopeFileSystem) {
  tokio::spawn(async move {
    stale_fs.ensure_exist().await?;

    let stale_versions = stale_fs
      .list_child()
      .await
      .unwrap_or_default()
      .into_iter()
      .filter(|child| Version::parse(child).is_some())
      .map(|child| stale_fs.child_fs(child));

    for stale_version in stale_versions {
      let _ = stale_version.remove().await;
    }

    Ok::<_, Error>(())
  });
}

async fn move_stale_versions(
  fs: &ScopeFileSystem,
  stale_fs: &ScopeFileSystem,
  stale_versions: Vec<Version>,
) -> Result<()> {
  stale_fs.ensure_exist().await?;

  for version in stale_versions {
    ScopeFileSystem::move_to(fs, stale_fs, version.as_str()).await?;
  }
  Ok(())
}

async fn refresh_metadata(
  fs: ScopeFileSystem,
  stale_fs: ScopeFileSystem,
  version: Version,
  expire: u64,
  max_versions: u32,
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
  let Ok((stale_versions, next_refresh_time)) =
    meta.refresh(&fs, &version, expire, max_versions).await
  else {
    return;
  };
  if meta.save(&fs).await.is_err() {
    return;
  }

  // Persist metadata before renaming directories so concurrent refreshes can
  // recover even if stale cleanup is interrupted.
  if move_stale_versions(&fs, &stale_fs, stale_versions)
    .await
    .is_err()
  {
    return;
  }
  spawn_cleanup_stale_versions(stale_fs);
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
  updates: StorageUpdates,
  /// Async tasks that delay collecting updates until save.
  update_tasks: Mutex<UpdateTasks>,
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
      db: DB::new(fs.child_fs(options.version.as_str())),
      task_queue: TaskQueue::default(),
      updates: Default::default(),
      update_tasks: Default::default(),
      next_meta_refresh_time: Default::default(),
      fs,
      options,
    }
  }

  pub fn stale_fs(&self) -> ScopeFileSystem {
    const STALE_DIR_NAME: &str = "_stale";
    self.fs.child_fs(STALE_DIR_NAME)
  }
}

#[async_trait::async_trait]
impl Storage for FileSystemStorage {
  fn cleanup_stale(&self) {
    spawn_cleanup_stale_versions(self.stale_fs());
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

  fn add_update_task(&mut self, task: StorageUpdateTask) {
    self
      .update_tasks
      .lock()
      .expect("should lock update tasks")
      .0
      .push(task);
  }

  fn save(&mut self) {
    // Take all pending updates and clear the memory buffer
    let updates = std::mem::take(&mut self.updates);
    let update_tasks = std::mem::take(
      &mut *self
        .update_tasks
        .get_mut()
        .expect("should get update tasks"),
    )
    .0;

    // Queue the write and metadata refresh together so cleanup observes the
    // latest version without blocking `save()`.
    let db = self.db.clone();
    let max_pack_size = self.options.max_pack_size;
    let fs = self.fs.clone();
    let stale_fs = self.stale_fs();
    let version = self.options.version.clone();
    let expire = self.options.expire;
    let max_versions = self.options.max_versions;
    let next_meta_refresh_time = self.next_meta_refresh_time.clone();

    self.task_queue.add_task(async move {
      let mut updates = updates;
      for task in update_tasks {
        for (scope, update) in task.await {
          updates.entry(scope).or_default().extend(update);
        }
      }
      let changes = updates
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();
      if db.save(changes, max_pack_size).await {
        refresh_metadata(
          fs,
          stale_fs,
          version,
          expire,
          max_versions,
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

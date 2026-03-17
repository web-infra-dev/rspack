mod db;
mod meta;
mod options;
mod scope_fs;

use std::sync::{Arc, Mutex};

use rustc_hash::FxHashMap as HashMap;

pub use self::options::FileSystemOptions;
use self::{db::DB, meta::Meta, scope_fs::ScopeFileSystem};
use crate::{Result, Storage};

/// Type alias for in-memory update changes: key -> optional_value
type BucketChangesMap = HashMap<Vec<u8>, Option<Vec<u8>>>;

/// File system-based persistent storage implementation
#[derive(Debug)]
pub struct FileSystemStorage {
  /// Root filesystem for metadata operations
  fs: ScopeFileSystem,
  /// Underlying database responsible for pack file read/write
  db: DB,
  /// In-memory staged update operations, grouped by scope
  /// Value of Some(value) indicates write, None indicates deletion
  updates: Mutex<HashMap<String, BucketChangesMap>>,
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

  async fn save(&self) -> Result<()> {
    // Take all pending updates and clear the memory buffer
    let updates = std::mem::take(&mut *self.updates.lock().expect("should get lock"));

    // Enqueue the write to the background task queue; errors are reported internally.
    // Call flush() to wait until the write has fully completed.
    self.db.save(
      updates
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect(),
      self.options.max_pack_size,
    );

    // Trigger metadata refresh in background (fire and forget)
    let fs = self.fs.clone();
    let version = self.options.version.clone();
    let expire = self.options.expire;
    let next_meta_refresh_time_lock = self.next_meta_refresh_time.clone();

    tokio::spawn(async move {
      // Check if it's time to refresh (check without holding lock across await)
      let now = Meta::current_timestamp();
      let should_refresh = {
        let next_time = next_meta_refresh_time_lock.lock().expect("should get lock");
        *next_time <= now
      }; // Lock dropped before async operations

      if !should_refresh {
        return;
      }

      // Perform refresh: load metadata (or create default) and update if needed
      let mut meta = match Meta::load(&fs).await {
        Ok(meta) => meta,
        Err(e) if e.is_not_found() => Default::default(),
        Err(_) => return,
      };
      if let Ok((expired_versions, next_refresh_time)) = meta.refresh(&version, expire).await {
        // Save updated metadata
        let _ = meta.save(&fs).await;

        // Remove expired version directories
        for v in expired_versions {
          let _ = fs.child_fs(&v).remove().await;
        }

        // Update next refresh time (short lock duration)
        let mut next_time = next_meta_refresh_time_lock.lock().expect("should get lock");
        *next_time = next_refresh_time;
      }
    });

    Ok(())
  }

  async fn reset(&self) {
    let _ = self.db.reset().await;
  }

  async fn flush(&self) {
    self.db.flush().await;
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    let names = self.db.bucket_names().await?;
    Ok(names)
  }
}

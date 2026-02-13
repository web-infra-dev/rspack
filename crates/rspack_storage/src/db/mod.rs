mod bucket;
mod error;
mod index;
mod meta;
mod options;
mod pack;
mod page;
mod transaction;

use std::sync::Arc;

use bucket::Bucket;
pub use error::{DBError, DBResult};
pub use options::DBOptions;
use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::RwLock;
pub use transaction::Transaction;

use crate::fs::FileSystem;

/// DB provides key-value storage with hot/cold separation
pub struct DB {
  root: Utf8PathBuf,
  options: DBOptions,
  fs: FileSystem,
  rw_lock: Arc<RwLock<()>>,
}

impl DB {
  pub fn new(
    root: Utf8PathBuf,
    options: DBOptions,
    fs: Arc<dyn IntermediateFileSystem>,
  ) -> DBResult<Self> {
    if options.page_count == 0 {
      return Err(DBError::InvalidFormat("page_count cannot be 0".to_string()));
    }

    Ok(Self {
      root,
      options,
      fs: FileSystem(fs),
      rw_lock: Default::default(),
    })
  }

  pub async fn load(&self, scope: &str) -> DBResult<HashMap<Vec<u8>, Vec<u8>>> {
    let _guard = self.rw_lock.read().await;

    let bucket = Bucket::new(
      scope.to_string(),
      &self.root,
      self.fs.clone(),
      self.options.clone(),
    );

    bucket.load().await
  }

  /// Save changes to the DB with two-phase locking
  ///
  /// # Two-Phase Locking Strategy
  /// - Phase 1 (Read Lock): Prepare data and write to .temp directory
  ///   - Load existing data
  ///   - Merge with changes
  ///   - Write prepared files to .temp/bucket/...
  /// - Phase 2 (Write Lock): Commit transaction
  ///   - Begin transaction (acquires state.lock)
  ///   - Move files from .temp to final locations
  ///   - Commit transaction (removes locks)
  ///
  /// This design minimizes write lock duration, allowing concurrent reads
  /// during the expensive data preparation phase.
  pub async fn save(
    &self,
    changes: HashMap<String, HashMap<Vec<u8>, Option<Vec<u8>>>>,
  ) -> DBResult<()> {
    let mut all_files_to_add = Vec::new();
    let mut all_files_to_remove = Vec::new();

    // Phase 1: Read lock - prepare data to .temp directory
    {
      let _guard = self.rw_lock.read().await;

      for (scope, scope_changes) in changes {
        let mut bucket = Bucket::new(scope, &self.root, self.fs.clone(), self.options.clone());

        let old_data = bucket.load().await.unwrap_or_default();
        let mut new_data = old_data;

        for (key, value) in scope_changes {
          match value {
            Some(v) => new_data.insert(key, v),
            None => new_data.remove(&key),
          };
        }

        let save_result = bucket.prepare_save(new_data).await?;
        all_files_to_add.extend(save_result.files_to_add);
        all_files_to_remove.extend(save_result.files_to_remove);
      }
    }

    // Phase 2: Write lock - commit from .temp to root
    {
      let _guard = self.rw_lock.write().await;

      let mut tx = Transaction::new(self.root.clone(), self.fs.clone());
      tx.begin().await?;

      for (path, content) in all_files_to_add {
        tx.add_file(&path, &content).await?;
      }

      for path in all_files_to_remove {
        tx.remove_file(&path);
      }

      tx.commit().await?;
    }

    Ok(())
  }
}

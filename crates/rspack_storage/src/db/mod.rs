mod bucket;
mod error;
mod options;
mod task_queue;
mod transaction;

use std::{collections::hash_map::Entry, sync::Arc};

use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::{
  Mutex,
  oneshot::{Receiver, channel},
};

use self::{bucket::Bucket, task_queue::TaskQueue, transaction::Transaction};
pub use self::{
  error::{Error, Result},
  options::Options,
};
use crate::{
  error::{Error as RootError, Result as RootResult},
  fs::{FileSystem, ScopeFileSystem},
};

/// Database providing key-value storage with hot/cold pack separation.
///
/// The DB organizes data into buckets, where each bucket contains multiple pack files
/// with automatic hot/cold separation for optimal performance.
#[derive(Debug)]
pub struct DB {
  options: Options,
  fs: ScopeFileSystem,
  /// Cached buckets, lazily loaded on first access
  buckets: Arc<Mutex<HashMap<String, Bucket>>>,
  /// Background task queue for asynchronous save operations
  task_queue: TaskQueue,
}

impl DB {
  /// Creates a new database instance at the specified root directory.
  pub fn new(root: Utf8PathBuf, options: Options, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    let fs = ScopeFileSystem::new(root.into_std_path_buf(), FileSystem(fs));
    Self {
      options,
      fs,
      buckets: Default::default(),
      task_queue: TaskQueue::default(),
    }
  }

  /// Returns a sorted list of all bucket names in the database.
  pub async fn bucket_names(&self) -> Result<Vec<String>> {
    self.fs.ensure_exist().await?;

    let entries = self.fs.list_child().await?;

    // Filter to keep only directories (buckets)
    let mut bucket_names = Vec::new();
    for entry in entries {
      if let Ok(metadata) = self.fs.stat(&entry).await {
        if metadata.is_directory {
          bucket_names.push(entry);
        }
      }
    }

    bucket_names.sort();
    Ok(bucket_names)
  }

  /// Loads all key-value pairs from the specified bucket.
  ///
  /// If the bucket doesn't exist yet, it will be created empty.
  pub async fn load(&self, bucket_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    self.fs.ensure_exist().await?;

    let mut buckets = self.buckets.lock().await;
    let bucket = match buckets.entry(bucket_name.to_string()) {
      Entry::Occupied(entry) => entry.into_mut(),
      Entry::Vacant(entry) => {
        let fs = self.fs.child_fs(bucket_name);
        let bucket = Bucket::new(fs, self.options.max_pack_size).await?;
        entry.insert(bucket)
      }
    };

    bucket.load_all().await
  }

  /// Saves changes to multiple buckets atomically using a two-phase commit.
  ///
  /// Changes are grouped by bucket name. For each key-value pair:
  /// - `Some(value)`: Set or update the key
  /// - `None`: Remove the key
  ///
  /// Returns a channel receiver that will report the save result asynchronously.
  pub fn save(
    &self,
    changes: HashMap<String, Vec<(Vec<u8>, Option<Vec<u8>>)>>,
  ) -> Result<Receiver<RootResult<()>>> {
    let (tx, rx) = channel();

    let mut all_files_to_add = Vec::new();
    let mut all_files_to_remove = Vec::new();

    let fs = self.fs.clone();
    let buckets = self.buckets.clone();
    let max_pack_size = self.options.max_pack_size;

    self.task_queue.add_task(async move {
      let task_fn = async move || -> Result<()> {
        let transaction = Transaction::new(&fs).await?;

        // Acquire write lock for the entire commit operation
        let mut buckets = buckets.lock().await;

        for (bucket_name, bucket_changes) in changes {
          // Get or create bucket
          let bucket = match buckets.entry(bucket_name.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
              let fs = transaction.readable_fs().child_fs(&bucket_name);
              let bucket = Bucket::new(fs, max_pack_size).await?;
              entry.insert(bucket)
            }
          };

          // Apply changes and collect file operations
          let writer_fs = transaction.writable_fs().child_fs(&bucket_name);
          let (added_pack, removed_pack) = bucket.save(Some(writer_fs), bucket_changes).await?;

          all_files_to_add.extend(
            added_pack
              .into_iter()
              .map(|file| format!("{bucket_name}/{file}")),
          );
          all_files_to_remove.extend(
            removed_pack
              .into_iter()
              .map(|file| format!("{bucket_name}/{file}")),
          );
        }

        // Atomically commit all changes
        transaction
          .commit(all_files_to_add, all_files_to_remove)
          .await
      };

      let _ = tx.send(task_fn().await.map_err(RootError::from));
    });

    Ok(rx)
  }

  /// Waits for all pending background save tasks to complete.
  pub async fn flush(&self) {
    self.task_queue.flush().await;
  }

  /// Removes the entire database from disk, deleting all buckets and data.
  pub async fn reset(&self) -> Result<()> {
    self.fs.remove().await?;
    Ok(())
  }
}

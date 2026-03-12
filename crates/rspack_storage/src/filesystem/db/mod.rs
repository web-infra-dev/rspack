mod bucket;
mod task_queue;
mod transaction;

use std::{collections::hash_map::Entry, sync::Arc};

use futures::future::try_join_all;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::{
  Mutex,
  oneshot::{Receiver, channel},
};

use self::{bucket::Bucket, task_queue::TaskQueue, transaction::Transaction};
use super::ScopeFileSystem;
use crate::{Error, Result};

/// Database providing key-value storage with hot/cold pack separation.
///
/// The DB organizes data into buckets, where each bucket contains multiple pack files
/// with automatic hot/cold separation for optimal performance.
#[derive(Debug)]
pub struct DB {
  max_pack_size: usize,
  fs: ScopeFileSystem,
  /// Cached buckets, lazily loaded on first access
  buckets: Arc<Mutex<HashMap<String, Bucket>>>,
  /// Background task queue for asynchronous save operations
  task_queue: TaskQueue,
}

impl DB {
  /// Creates a new database instance at the specified root directory.
  pub fn new(max_pack_size: usize, fs: ScopeFileSystem) -> Self {
    Self {
      max_pack_size,
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
        let bucket = Bucket::new(fs, self.max_pack_size).await?;
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
  ) -> Result<Receiver<Result<()>>> {
    let (tx, rx) = channel();

    let fs = self.fs.clone();
    let buckets = self.buckets.clone();
    let max_pack_size = self.max_pack_size;

    self.task_queue.add_task(async move {
      let task_fn = async move || -> Result<()> {
        let transaction = Transaction::new(&fs).await?;

        // Acquire write lock for the entire commit operation
        let mut buckets = buckets.lock().await;

        let mut pending_buckets = Vec::with_capacity(changes.len());
        for (bucket_name, bucket_changes) in changes {
          let bucket = if let Some(bucket) = buckets.remove(&bucket_name) {
            bucket
          } else {
            let fs = transaction.readable_fs().child_fs(&bucket_name);
            Bucket::new(fs, max_pack_size).await?
          };
          pending_buckets.push((bucket_name, bucket, bucket_changes));
        }

        let results = try_join_all(pending_buckets.into_iter().map(
          |(bucket_name, mut bucket, changes)| {
            // Apply changes and collect file operations
            let writable_fs = transaction.writable_fs().child_fs(&bucket_name);
            async move {
              let affacted_files = bucket.save(Some(writable_fs), changes).await?;
              Ok::<_, Error>((bucket_name, bucket, affacted_files))
            }
          },
        ))
        .await?;

        let mut all_files_to_add = Vec::new();
        let mut all_files_to_remove = Vec::new();
        for (bucket_name, bucket, affacted_files) in results {
          let (added_pack, removed_pack) = affacted_files;
          buckets.insert(bucket_name.clone(), bucket);
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

      let _ = tx.send(task_fn().await.map_err(Error::from));
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

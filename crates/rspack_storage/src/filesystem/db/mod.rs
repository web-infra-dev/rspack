mod bucket;
mod task_queue;
mod transaction;

use std::{
  collections::hash_map::Entry,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use rspack_parallel::TryFutureConsumer;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::Mutex;

use self::{bucket::Bucket, task_queue::TaskQueue, transaction::Transaction};
use super::ScopeFileSystem;
use crate::{Error, Result};

/// Type alias for bucket changes: bucket_name -> Vec<(key, optional_value)>
type BucketChanges = HashMap<String, Vec<(Vec<u8>, Option<Vec<u8>>)>>;

/// Database providing key-value storage with hot/cold pack separation.
///
/// The DB organizes data into buckets, where each bucket contains multiple pack files
/// with automatic hot/cold separation for optimal performance.
#[derive(Debug)]
pub struct DB {
  fs: ScopeFileSystem,
  /// Cached buckets, lazily loaded on first access
  buckets: Arc<Mutex<HashMap<String, Bucket>>>,
  /// Background task queue for asynchronous save operations
  task_queue: Arc<TaskQueue>,
  readonly: Arc<AtomicBool>,
}

impl DB {
  /// Creates a new database instance at the specified root directory.
  pub fn new(fs: ScopeFileSystem) -> Self {
    Self {
      fs,
      buckets: Default::default(),
      task_queue: Arc::new(TaskQueue::default()),
      readonly: Arc::new(AtomicBool::new(false)),
    }
  }

  /// Returns a sorted list of all bucket names in the database.
  pub async fn bucket_names(&self) -> Result<Vec<String>> {
    self.fs.ensure_exist().await?;

    let entries = self.fs.list_child().await?;

    // Filter to keep only directories (buckets), excluding internal directories
    let mut bucket_names = Vec::new();
    for entry in entries {
      if !entry.starts_with('.')
        && let Ok(metadata) = self.fs.stat(&entry).await
        && metadata.is_directory
      {
        bucket_names.push(entry);
      }
    }

    bucket_names.sort();
    Ok(bucket_names)
  }

  /// Loads all key-value pairs from the specified bucket.
  pub async fn load(&self, bucket_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut buckets = self.buckets.lock().await;

    self.fs.ensure_exist().await?;
    // Ensure any crashed prior transaction is recovered before we read.
    Transaction::ensure_committed(&self.fs).await?;

    let bucket = match buckets.entry(bucket_name.to_string()) {
      Entry::Occupied(entry) => entry.into_mut(),
      Entry::Vacant(entry) => {
        let fs = self.fs.child_fs(bucket_name);
        let bucket = Bucket::new(fs).await?;
        entry.insert(bucket)
      }
    };

    bucket.load_all().await
  }

  /// Enqueues changes to multiple buckets for atomic persistence using a two-phase commit.
  ///
  /// Changes are grouped by bucket name. For each key-value pair:
  /// - `Some(value)`: Set or update the key
  /// - `None`: Remove the key
  ///
  /// No-op when the DB is in readonly mode.
  pub fn save(&self, changes: BucketChanges, max_pack_size: usize) {
    let fs = self.fs.clone();
    let buckets = self.buckets.clone();
    let readonly = self.readonly.clone();

    self.task_queue.add_task(async move {
      if readonly.load(Ordering::Relaxed) {
        return;
      }

      let task_fn = async move || -> Result<()> {
        let mut buckets = buckets.lock().await;

        let transaction = Transaction::new(&fs).await?;

        // Split changes into cached vs. uncached buckets so Bucket::new and
        // bucket.save can run concurrently across buckets.
        let pending_buckets: Vec<_> = changes
          .into_iter()
          .map(|(bucket_name, bucket_changes)| {
            let cached = buckets.remove(&bucket_name);
            (bucket_name, cached, bucket_changes)
          })
          .collect();

        let mut all_files_to_add = Vec::new();
        let mut all_files_to_remove = Vec::new();
        let mut updated_buckets = HashMap::default();
        let save_result = pending_buckets
          .into_iter()
          .map(|(bucket_name, cached_bucket, changes)| {
            let readable_fs = transaction.readable_fs().child_fs(&bucket_name);
            let writable_fs = transaction.writable_fs().child_fs(&bucket_name);
            async move {
              // Initialize bucket if not already cached (runs in parallel across buckets)
              let mut bucket = if let Some(bucket) = cached_bucket {
                bucket
              } else {
                Bucket::new(readable_fs).await?
              };
              let affacted_files = bucket
                .save(Some(writable_fs), changes, max_pack_size)
                .await?;
              Ok::<_, Error>((bucket_name, bucket, affacted_files))
            }
          })
          .try_fut_consume(|(bucket_name, bucket, affacted_files)| {
            let (added_pack, removed_pack) = affacted_files;
            updated_buckets.insert(bucket_name.clone(), bucket);
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
          })
          .await;

        match save_result {
          Ok(()) => {
            transaction
              .commit(all_files_to_add, all_files_to_remove)
              .await?;
            buckets.extend(updated_buckets);
          }
          Err(e) => {
            transaction.rollback().await?;
            return Err(e);
          }
        }
        Ok(())
      };

      if let Err(err) = task_fn().await {
        // The cache may be in an indeterminate state. Switch to readonly so no
        // further writes can make things worse. Restart the process to recover.
        // The current build is not affected.
        readonly.store(true, Ordering::Relaxed);
        println!(
          "Rspack persistent cache save failed: {err}\n  \
           Persistent cache has been disabled for this session. \
           Restart the process to re-enable it."
        );
      }
    });
  }

  /// Waits for all pending background save tasks to complete.
  pub async fn flush(&self) {
    self.task_queue.flush().await;
  }

  /// Removes the entire database from disk, deleting all buckets and data.
  pub async fn reset(&self) -> Result<()> {
    self.flush().await;
    self.buckets.lock().await.clear();
    self.fs.remove().await?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::{DB, HashMap, Result, ScopeFileSystem};

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_db() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/".into());
    let db = DB::new(fs);
    let name_1 = "name1";
    let name_2 = "name2";
    assert!(db.bucket_names().await?.is_empty());
    assert!(db.load(name_1).await?.is_empty());

    let bucket_data: Vec<_> = (0..9)
      .map(|num| {
        (
          format!("key{num}").as_bytes().to_vec(),
          Some(format!("value{num}").as_bytes().to_vec()),
        )
      })
      .collect();

    let mut data = HashMap::default();
    data.insert(String::from(name_1), bucket_data.clone());
    data.insert(String::from(name_2), bucket_data);
    // save data and wait finish
    db.save(data, 25);
    db.flush().await;

    let mut data1 = db.load(name_1).await?;
    data1.sort();
    let mut data2 = db.load(name_2).await?;
    data2.sort();
    assert_eq!(data1.len(), 9);
    assert_eq!(data1, data2);

    let mut names = db.bucket_names().await?;
    names.sort();
    assert_eq!(names, vec![String::from(name_1), String::from(name_2)]);

    db.reset().await?;
    assert!(db.bucket_names().await?.is_empty());
    assert!(db.load(name_1).await?.is_empty());

    Ok(())
  }
}

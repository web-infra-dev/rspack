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

/// DB provides key-value storage with hot/cold separation
#[derive(Debug)]
pub struct DB {
  options: Options,
  fs: ScopeFileSystem,
  transaction: Arc<Transaction>,
  buckets: Arc<Mutex<HashMap<String, Bucket>>>,
  task_queue: TaskQueue,
}

impl DB {
  pub fn new(root: Utf8PathBuf, options: Options, fs: Arc<dyn IntermediateFileSystem>) -> Self {
    let fs = ScopeFileSystem::new(root.into_std_path_buf(), FileSystem(fs));
    Self {
      options,
      transaction: Arc::new(Transaction::new(&fs)),
      fs,
      buckets: Default::default(),
      task_queue: TaskQueue::new(),
    }
  }

  pub async fn init(&self) -> Result<()> {
    self.transaction.init().await
  }

  pub async fn bucket_names(&self) -> Result<Vec<String>> {
    // Read all entries in the root directory
    let entries = self.fs.list_child().await?;

    // Filter to keep only directories (buckets)
    let mut bucket_names = Vec::new();
    for entry in entries {
      // Check if it's a directory by trying to stat it
      if let Ok(metadata) = self.fs.stat(&entry).await {
        if metadata.is_directory {
          bucket_names.push(entry);
        }
      }
    }

    bucket_names.sort();
    Ok(bucket_names)
  }

  pub async fn load(&self, bucket_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
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

  pub fn save(
    &self,
    changes: HashMap<String, Vec<(Vec<u8>, Option<Vec<u8>>)>>,
  ) -> Result<Receiver<RootResult<()>>> {
    let (tx, rx) = channel();
    let transaction = self.transaction.clone();
    let temp_fs = transaction.temp_fs.clone();

    let mut all_files_to_add = Vec::new();
    let mut all_files_to_remove = Vec::new();

    let buckets = self.buckets.clone();
    let max_pack_size = self.options.max_pack_size;
    self.task_queue.add_task(async move {
      let task_fn = async move || {
        // Acquire write lock during commit
        let mut buckets = buckets.lock().await;
        for (bucket_name, bucket_changes) in changes {
          let bucket = match buckets.entry(bucket_name.to_string()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
              let fs = transaction.root_fs.child_fs(&bucket_name);
              let bucket = Bucket::new(fs, max_pack_size).await?;
              entry.insert(bucket)
            }
          };
          let (added_pack, removed_pack) =
            bucket.save(Some(temp_fs.clone()), bucket_changes).await?;
          all_files_to_add.extend(added_pack);
          all_files_to_remove.extend(removed_pack);
        }
        // Commit from .temp to root
        transaction
          .commit(all_files_to_add, all_files_to_remove)
          .await
      };
      let _ = tx.send(task_fn().await.map_err(|e| RootError::from(e)));
    });

    Ok(rx)
  }

  /// Wait for all pending background tasks to complete
  pub async fn flush(&self) {
    self.task_queue.flush().await;
  }

  pub async fn reset(&self) -> Result<()> {
    self.fs.clean().await?;
    Ok(())
  }
}

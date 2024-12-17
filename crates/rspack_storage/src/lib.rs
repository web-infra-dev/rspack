mod error;
mod fs;
mod pack;

use std::sync::Arc;

pub use error::StorageResult;
pub use fs::{
  StorageBridgeFS, StorageFS, StorageFSError, StorageFSOperation, StorageFSResult, StorageReader,
  StorageWriter,
};
pub use pack::{PackStorage, PackStorageOptions};
use tokio::sync::oneshot::Receiver;

type StorageItemKey = Vec<u8>;
type StorageItemValue = Vec<u8>;
type StorageContent = Vec<(Arc<StorageItemKey>, Arc<StorageItemValue>)>;

#[async_trait::async_trait]
pub trait Storage: std::fmt::Debug + Sync + Send {
  async fn load(&self, scope: &'static str) -> StorageResult<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>>;
  fn set(&self, scope: &'static str, key: Vec<u8>, value: Vec<u8>);
  fn remove(&self, scope: &'static str, key: &[u8]);
  fn trigger_save(&self) -> StorageResult<Receiver<StorageResult<()>>>;
}

pub type ArcStorage = Arc<dyn Storage>;

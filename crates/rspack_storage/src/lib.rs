mod db;
mod error;
mod fs;
mod local_storage;
mod pack;

use std::sync::Arc;

pub use db::DB;
pub use error::Result;
pub use fs::{FSError, FSOperation, FSResult, FileSystem, Reader, Writer};
pub use local_storage::LocalStorage;
pub use pack::{PackStorage as A, PackStorageOptions};
use tokio::sync::oneshot::Receiver;

pub type PackStorage = self::local_storage::LocalStorage;

type ItemKey = Vec<u8>;
type ItemValue = Vec<u8>;
type ItemPairs = Vec<(Arc<ItemKey>, Arc<ItemValue>)>;

#[async_trait::async_trait]
pub trait Storage: std::fmt::Debug + Sync + Send {
  async fn load(&self, scope: &'static str) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>>;
  fn set(&self, scope: &'static str, key: Vec<u8>, value: Vec<u8>);
  fn remove(&self, scope: &'static str, key: &[u8]);
  fn trigger_save(&self) -> Result<Receiver<Result<()>>>;
  async fn reset(&self);
  /// Get list of all available scopes in the storage
  async fn scopes(&self) -> Result<Vec<String>>;
}

pub type ArcStorage = Arc<dyn Storage>;

mod error;
mod filesystem;
mod memory;

use std::sync::Arc;

use tokio::sync::oneshot::Receiver;

pub use self::{
  error::{Error, Result},
  filesystem::{FileSystemOptions, FileSystemStorage},
};

pub type Key = Vec<u8>;
pub type Value = Vec<u8>;
pub type KVPairs<V = Value> = Vec<(Key, V)>;

#[async_trait::async_trait]
pub trait Storage: std::fmt::Debug + Sync + Send {
  async fn load(&self, scope: &'static str) -> Result<KVPairs>;
  fn set(&self, scope: &'static str, key: Key, value: Value);
  fn remove(&self, scope: &'static str, key: &[u8]);
  fn trigger_save(&self) -> Result<Receiver<Result<()>>>;
  async fn reset(&self);
  /// Get list of all available scopes in the storage
  async fn scopes(&self) -> Result<Vec<String>>;
}

pub type ArcStorage = Arc<dyn Storage>;

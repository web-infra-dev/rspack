mod fs;

use std::sync::Arc;

pub use fs::FsStorage;

// 内存缓存 与 持久化缓存 同步
// 不用考虑失败的情况

// Storage 考虑一致性
pub trait Storage: std::fmt::Debug + Sync + Send {
  fn get_all(&self, scope: &'static str) -> Vec<(Vec<u8>, Vec<u8>)>;
  //  fn get(&self, scope: &str, key: &[u8]) -> Option<Vec<u8>>;
  // using immutable reference to support concurrency
  fn set(&self, scope: &'static str, key: Vec<u8>, value: Vec<u8>);
  fn remove(&self, scope: &'static str, key: &[u8]);
  fn idle(&self);
}

pub type ArcStorage = Arc<dyn Storage>;

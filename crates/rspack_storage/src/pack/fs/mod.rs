use rspack_error::Result;

mod native;
pub use native::PackNativeFs;

mod memory;
pub use memory::PackMemoryFs;

mod error;
pub use error::{PackFsError, PackFsErrorOpt};
use rspack_paths::Utf8Path;
use rustc_hash::FxHashSet as HashSet;

pub struct FileMeta {
  pub size: u64,
  pub mtime: u64,
  pub is_file: bool,
  pub is_dir: bool,
}

#[async_trait::async_trait]
pub trait PackFileReader: std::fmt::Debug + Sync + Send {
  async fn line(&mut self) -> Result<String>;
  async fn bytes(&mut self, len: usize) -> Result<Vec<u8>>;
  async fn skip(&mut self, len: usize) -> Result<()>;
  async fn remain(&mut self) -> Result<Vec<u8>>;
}

#[async_trait::async_trait]
pub trait PackFileWriter: std::fmt::Debug + Sync + Send {
  async fn line(&mut self, line: &str) -> Result<()>;
  async fn bytes(&mut self, bytes: &[u8]) -> Result<()>;
  async fn flush(&mut self) -> Result<()>;
  async fn write(&mut self, content: &[u8]) -> Result<()>;
}

#[async_trait::async_trait]
pub trait PackFs: std::fmt::Debug + Sync + Send {
  async fn exists(&self, path: &Utf8Path) -> Result<bool>;
  async fn remove_dir(&self, path: &Utf8Path) -> Result<()>;
  async fn ensure_dir(&self, path: &Utf8Path) -> Result<()>;
  async fn write_file(&self, path: &Utf8Path) -> Result<Box<dyn PackFileWriter>>;
  async fn read_file(&self, path: &Utf8Path) -> Result<Box<dyn PackFileReader>>;
  async fn read_dir(&self, path: &Utf8Path) -> Result<HashSet<String>>;
  async fn metadata(&self, path: &Utf8Path) -> Result<FileMeta>;
  async fn remove_file(&self, path: &Utf8Path) -> Result<()>;
  async fn move_file(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()>;
}

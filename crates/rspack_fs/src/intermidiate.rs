use std::fmt::Debug;

use rspack_paths::Utf8Path;

use super::Result;

#[async_trait::async_trait]
pub trait IntermediateFileSystemExtras: Debug + Send + Sync {
  async fn rename(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()>;
  async fn create_read_stream(&self, file: &Utf8Path) -> Result<Box<dyn ReadStream>>;
  async fn create_write_stream(&self, file: &Utf8Path) -> Result<Box<dyn WriteStream>>;
}

#[async_trait::async_trait]
pub trait ReadStream: Debug + Sync + Send {
  async fn read(&mut self, buf: &mut [u8]) -> Result<()>;
  async fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize>;
  async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize>;
  async fn skip(&mut self, offset: usize) -> Result<()>;
  async fn close(&mut self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait WriteStream: Debug + Sync + Send {
  async fn write(&mut self, buf: &[u8]) -> Result<usize>;
  async fn write_all(&mut self, buf: &[u8]) -> Result<()>;
  async fn flush(&mut self) -> Result<()>;
  async fn close(&mut self) -> Result<()>;
}

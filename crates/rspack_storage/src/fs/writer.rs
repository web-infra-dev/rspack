use rspack_fs::WriteStream;
use rspack_paths::Utf8PathBuf;

use super::{FSOperation, FSResult, error::FsResultToStorageFsResult};

#[derive(Debug)]
pub struct Writer {
  pub path: Utf8PathBuf,
  pub stream: Box<dyn WriteStream>,
}

impl Writer {
  pub async fn write_line(&mut self, line: &str) -> FSResult<()> {
    self
      .stream
      .write_line(line)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn write(&mut self, buf: &[u8]) -> FSResult<usize> {
    self
      .stream
      .write(buf)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn write_all(&mut self, buf: &[u8]) -> FSResult<()> {
    self
      .stream
      .write_all(buf)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn flush(&mut self) -> FSResult<()> {
    self
      .stream
      .flush()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
  pub async fn close(&mut self) -> FSResult<()> {
    self
      .stream
      .close()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Write)
  }
}

use rspack_fs::ReadStream;
use rspack_paths::Utf8PathBuf;

use super::{FSOperation, FSResult, error::FsResultToStorageFsResult};

#[derive(Debug)]
pub struct Reader {
  pub path: Utf8PathBuf,
  pub stream: Box<dyn ReadStream>,
}

impl Reader {
  pub async fn read_line(&mut self) -> FSResult<String> {
    self
      .stream
      .read_line()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn read(&mut self, length: usize) -> FSResult<Vec<u8>> {
    self
      .stream
      .read(length)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn read_until(&mut self, byte: u8) -> FSResult<Vec<u8>> {
    self
      .stream
      .read_until(byte)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn read_to_end(&mut self) -> FSResult<Vec<u8>> {
    self
      .stream
      .read_to_end()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn skip(&mut self, offset: usize) -> FSResult<()> {
    self
      .stream
      .skip(offset)
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
  pub async fn close(&mut self) -> FSResult<()> {
    self
      .stream
      .close()
      .await
      .to_storage_fs_result(&self.path, FSOperation::Read)
  }
}

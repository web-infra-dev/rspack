use std::sync::Arc;

use rspack_fs::IntermediateFileSystem;
use rspack_paths::Utf8PathBuf;

#[derive(Debug)]
pub struct FileSystemOptions {
  pub directory: Utf8PathBuf,
  pub max_pack_size: usize,
  pub expire: u64,
  pub fs: Arc<dyn IntermediateFileSystem>,
}

/*impl Default for FileSystemOptions {
  fn default() -> Self {
    Self {
      directory: Utf8PathBuf::new(),
      max_pack_size: 512 * 1024,
      expire: 0,
    }
  }
}*/

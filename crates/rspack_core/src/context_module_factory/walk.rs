use std::sync::Arc;

use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{Utf8Path, Utf8PathBuf};

/// Walk a directory tree recursively, calling `on_file` for each file found.
///
/// - `root`: starting directory
/// - `recursive`: whether to descend into subdirectories
/// - `skip_dotfiles`: whether to skip files whose name starts with `.`
/// - `should_enter_dir`: called with (full_path, dirname) for each directory before descending
/// - `on_file`: called with (full_path, filename) for each file
#[async_recursion]
pub(crate) async fn walk_dir(
  root: &Utf8Path,
  fs: Arc<dyn ReadableFileSystem>,
  recursive: bool,
  skip_dotfiles: bool,
  should_enter_dir: &mut (impl FnMut(&Utf8Path, &str) -> bool + Send),
  on_file: &mut (impl FnMut(Utf8PathBuf, String) + Send),
) -> Result<()> {
  if !fs.metadata(root).await.is_ok_and(|m| m.is_directory) {
    return Ok(());
  }
  for filename in fs.read_dir(root).await? {
    let path = root.join(&filename);
    if fs.metadata(&path).await.is_ok_and(|m| m.is_directory) {
      if recursive && should_enter_dir(&path, &filename) {
        walk_dir(
          &path,
          fs.clone(),
          recursive,
          skip_dotfiles,
          should_enter_dir,
          on_file,
        )
        .await?;
      }
    } else if skip_dotfiles && filename.starts_with('.') {
      // skip dotfiles
    } else {
      on_file(path, filename);
    }
  }
  Ok(())
}

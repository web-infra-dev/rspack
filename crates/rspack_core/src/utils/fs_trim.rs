use rspack_fs::Result;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8Path;

/// Remove all files and directories in the given directory except the given directory
///
/// example:
/// ```
/// #[tokio::test]
/// async fn remove_dir_except() {
///   use crate::rspack_fs::ReadableFileSystem;
///   use crate::rspack_fs::WritableFileSystem;
///   use crate::rspack_fs::WritableFileSystemExt;
///   use crate::rspack_paths::Utf8Path;
///   let fs = crate::rspack_fs::NativeFileSystem;
///
///   // adding files and directories
///   fs.create_dir_all(&Utf8Path::new("path/to/dir/except"))
///     .await
///     .unwrap();
///   fs.create_dir_all(&Utf8Path::new("path/to/dir/rm1"))
///     .await
///     .unwrap();
///   fs.create_dir_all(&Utf8Path::new("path/to/dir/rm2"))
///     .await
///     .unwrap();
///
///   let dir = Utf8Path::new("path/to/dir");
///   let except = Utf8Path::new("path/to/dir/except");
///
///   trim_dir(fs, &dir, &except).await.unwrap();
///   assert_eq!(
///     fs.read_dir(&dir).await.unwrap(),
///     vec![String::from("path/to/dir/except")]
///   );
///
///   fs.remove_dir_all(&dir).await.unwrap();
/// }
/// ```
pub async fn trim_dir<'a>(
  fs: &'a dyn WritableFileSystem,
  dir: &'a Utf8Path,
  except: &'a Utf8Path,
) -> Result<()> {
  if dir.starts_with(except) {
    return Ok(());
  }
  if !except.starts_with(dir) {
    return fs.remove_dir_all(dir).await;
  }

  let mut to_clean = dir;
  while to_clean != except {
    let mut matched = None;
    for entry in fs.read_dir(dir).await? {
      let path = dir.join(entry);
      if except.starts_with(&path) {
        matched = Some(except);
        continue;
      }
      if fs.stat(&path).await?.is_directory {
        fs.remove_dir_all(&path).await?;
      } else {
        fs.remove_file(&path).await?;
      }
    }
    let Some(child_to_clean) = matched else {
      break;
    };
    to_clean = child_to_clean;
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::Utf8Path;

  use crate::trim_dir;

  #[tokio::test]
  async fn async_fs_test() {
    let fs = MemoryFileSystem::default();
    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/ex/a1/b1"))
        .await
        .is_ok()
    );

    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/ex/a2/b1"))
        .await
        .is_ok()
    );

    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/ex/a2/b2"))
        .await
        .is_ok()
    );

    assert!(
      WritableFileSystem::create_dir_all(&fs, Utf8Path::new("/ex/a3/b1"))
        .await
        .is_ok()
    );

    assert!(trim_dir(&fs, Utf8Path::new("/ex"), Utf8Path::new("/ex/a2"))
      .await
      .is_ok());

    let children = WritableFileSystem::read_dir(&fs, Utf8Path::new("/ex"))
      .await
      .unwrap();
    assert_eq!(children, vec!["a2"]);
  }
}

use cow_utils::CowUtils;
use rspack_error::Result;
use rspack_fs::{Result as FsResult, WritableFileSystem};
use rspack_paths::Utf8Path;
use rspack_regex::RspackRegex;

use crate::KeepFunc;

pub enum KeepPattern<'a> {
  Path(&'a Utf8Path),
  Regex(&'a RspackRegex),
  Func(&'a KeepFunc),
}

impl<'a> KeepPattern<'a> {
  pub async fn try_match(&self, path: &'a Utf8Path) -> Result<bool> {
    match self {
      KeepPattern::Path(p) => Ok(path.starts_with(p)),
      KeepPattern::Regex(r) => Ok(r.test(path.as_str().cow_replace("\\", "/").as_ref())),
      KeepPattern::Func(f) => f(path.as_str().cow_replace("\\", "/").to_string()).await,
    }
  }
}

/// Remove all files and directories in the given directory except the given directory
///
/// example:
/// ```
/// #[tokio::test]
/// async fn remove_dir_except() {
///   use crate::{
///     rspack_fs::{ReadableFileSystem, WritableFileSystem, WritableFileSystemExt},
///     rspack_paths::Utf8Path,
///   };
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
  keep: KeepPattern<'a>,
) -> FsResult<()> {
  if let Ok(metadata) = fs.stat(dir).await {
    // not a directory, try to remove it
    if !metadata.is_directory {
      if !keep.try_match(dir).await? {
        fs.remove_file(dir).await?;
      }
      return Ok(());
    }
  } else {
    // not exists, no need to trim
    return Ok(());
  }

  let mut queue = vec![dir.to_owned()];
  let mut visited = vec![];
  while let Some(current_dir) = queue.pop() {
    if keep.try_match(&current_dir).await? {
      visited.push(current_dir);
      continue;
    }
    let items = fs.read_dir(&current_dir).await?;
    for item in &items {
      let path = current_dir.join(item);
      if fs.stat(&path).await?.is_directory {
        queue.push(path);
      } else if !keep.try_match(&path).await? {
        fs.remove_file(&path).await?;
      }
    }

    visited.push(current_dir);
  }

  visited.reverse();
  for dir in visited {
    if fs.read_dir(&dir).await?.is_empty() {
      fs.remove_dir_all(&dir).await?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod test {
  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::Utf8Path;

  use crate::{trim_dir, KeepPattern};

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

    assert!(trim_dir(
      &fs,
      Utf8Path::new("/ex"),
      KeepPattern::Path(Utf8Path::new("/ex/a2"))
    )
    .await
    .is_ok());

    let children = WritableFileSystem::read_dir(&fs, Utf8Path::new("/ex"))
      .await
      .unwrap();
    assert_eq!(children, vec!["a2"]);
  }
}

use cow_utils::CowUtils;
use rspack_error::Result;
use rspack_fs::{AbsPath, Result as FsResult, WritableFileSystem};
use rspack_paths::Utf8Path;
use rspack_regex::RspackRegex;

use crate::KeepFunc;

pub enum KeepPattern<'a> {
  Path(&'a Utf8Path),
  Regex(&'a RspackRegex),
  Func(&'a KeepFunc),
}

impl<'a> KeepPattern<'a> {
  pub async fn try_match(&self, path: &Utf8Path) -> Result<bool> {
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
///     rspack_fs::{AbsPath, NativeFileSystem, WritableFileSystem},
///     rspack_paths::Utf8Path,
///   };
///   let fs = NativeFileSystem::new(false);
///
///   // adding files and directories
///   fs.create_dir_all(&AbsPath::new("/path/to/dir/except").unwrap())
///     .await
///     .unwrap();
///   fs.create_dir_all(&AbsPath::new("/path/to/dir/rm1").unwrap())
///     .await
///     .unwrap();
///   fs.create_dir_all(&AbsPath::new("/path/to/dir/rm2").unwrap())
///     .await
///     .unwrap();
///
///   let dir = Utf8Path::new("/path/to/dir");
///   let except = Utf8Path::new("/path/to/dir/except");
///
///   trim_dir(&fs, dir, KeepPattern::Path(except)).await.unwrap();
///   assert_eq!(
///     fs.read_dir(&AbsPath::try_from_utf8_path(dir).unwrap())
///       .await
///       .unwrap(),
///     vec![String::from("except")]
///   );
///
///   fs.remove_dir_all(&AbsPath::try_from_utf8_path(dir).unwrap())
///     .await
///     .unwrap();
/// }
/// ```
pub async fn trim_dir<'a>(
  fs: &'a dyn WritableFileSystem,
  dir: &'a Utf8Path,
  keep: KeepPattern<'a>,
) -> FsResult<()> {
  let abs_dir = AbsPath::try_from_utf8_path(dir)?;
  if let Ok(metadata) = fs.stat(&abs_dir).await {
    // not a directory, try to remove it
    if !metadata.is_directory {
      if !keep.try_match(abs_dir.as_utf8_path()).await? {
        fs.remove_file(&abs_dir).await?;
      }
      return Ok(());
    }
  } else {
    // not exists, no need to trim
    return Ok(());
  }

  let mut queue = vec![abs_dir];
  let mut visited = vec![];
  while let Some(current_dir) = queue.pop() {
    if keep.try_match(current_dir.as_utf8_path()).await? {
      visited.push(current_dir);
      continue;
    }
    let items = fs.read_dir(&current_dir).await?;
    for item in &items {
      let path = current_dir.join(item)?;
      if fs.stat(&path).await?.is_directory {
        queue.push(path);
      } else if !keep.try_match(path.as_utf8_path()).await? {
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
  use cow_utils::CowUtils;
  use rspack_fs::{AbsPath, MemoryFileSystem, WritableFileSystem};
  use rspack_paths::Utf8Path;

  use crate::{KeepPattern, trim_dir};

  fn test_str(posix: &str) -> String {
    if cfg!(windows) {
      let rest = posix.trim_start_matches('/').cow_replace('/', "\\");
      format!("C:\\{rest}")
    } else {
      posix.to_string()
    }
  }

  fn test_abs(posix: &str) -> AbsPath {
    AbsPath::new(&test_str(posix)).expect("valid test AbsPath")
  }

  #[tokio::test]
  async fn async_fs_test() {
    let fs = MemoryFileSystem::default();
    assert!(
      WritableFileSystem::create_dir_all(&fs, &test_abs("/ex/a1/b1"))
        .await
        .is_ok()
    );

    assert!(
      WritableFileSystem::create_dir_all(&fs, &test_abs("/ex/a2/b1"))
        .await
        .is_ok()
    );

    assert!(
      WritableFileSystem::create_dir_all(&fs, &test_abs("/ex/a2/b2"))
        .await
        .is_ok()
    );

    assert!(
      WritableFileSystem::create_dir_all(&fs, &test_abs("/ex/a3/b1"))
        .await
        .is_ok()
    );

    let ex = test_str("/ex");
    let keep = test_str("/ex/a2");
    assert!(
      trim_dir(
        &fs,
        Utf8Path::new(&ex),
        KeepPattern::Path(Utf8Path::new(&keep))
      )
      .await
      .is_ok()
    );

    let children = WritableFileSystem::read_dir(&fs, &test_abs("/ex"))
      .await
      .unwrap();
    assert_eq!(children, vec!["a2"]);
  }
}

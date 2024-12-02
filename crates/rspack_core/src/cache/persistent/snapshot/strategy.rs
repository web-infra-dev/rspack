use std::{
  path::Path,
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use rspack_cacheable::cacheable;
use rspack_fs::FileSystem;
use rspack_paths::{ArcPath, AssertUtf8};
use rustc_hash::FxHashMap as HashMap;

/// Snapshot check strategy
#[cacheable]
#[derive(Debug, PartialEq)]
pub enum Strategy {
  /// Check by package version
  ///
  /// This strategy will find the package.json in the parent directory, and
  /// compares the version field.
  PackageVersion(String),

  /// Check by compile time
  ///
  /// This strategy will compare the compile time and the file update time
  CompileTime(u64),
}

/// Validate Result
#[derive(Debug)]
pub enum ValidateResult {
  /// The target file has been deleted
  Deleted,
  /// The target file has been modified
  Modified,
  /// The target file has no changed
  NoChanged,
}

pub struct StrategyHelper {
  fs: Arc<dyn FileSystem>,
  package_version_cache: HashMap<ArcPath, Option<String>>,
}

impl StrategyHelper {
  pub fn new(fs: Arc<dyn FileSystem>) -> Self {
    Self {
      fs,
      package_version_cache: Default::default(),
    }
  }

  /// get path file modified time
  fn modified_time(&self, path: &Path) -> Option<u64> {
    if let Ok(info) = self.fs.metadata(path.assert_utf8()) {
      Some(info.mtime_ms)
    } else {
      None
    }
  }

  /// get path file version in package.json
  fn package_version_with_cache(&mut self, path: &Path) -> Option<String> {
    if let Some(version) = self.package_version_cache.get(path) {
      return version.clone();
    }

    let mut res = None;
    if let Ok(content) = self.fs.read(&path.join("package.json").assert_utf8()) {
      if let Ok(mut package_json) =
        serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&content)
      {
        if let Some(serde_json::Value::String(version)) = package_json.remove("version") {
          res = Some(version);
        }
      }
    }

    if res.is_none() {
      if let Some(p) = path.parent() {
        res = self.package_version_with_cache(p);
      }
    }

    self.package_version_cache.insert(path.into(), res.clone());
    res
  }

  /// get current time as compile time strategy
  pub fn compile_time() -> Strategy {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("get current time failed")
      .as_millis() as u64;
    Strategy::CompileTime(now)
  }
  /// get path file package version strategy
  pub fn package_version(&mut self, path: &Path) -> Option<Strategy> {
    self
      .package_version_with_cache(path)
      .map(Strategy::PackageVersion)
  }

  /// validate path file by target strategy
  pub fn validate(&mut self, path: &Path, strategy: &Strategy) -> ValidateResult {
    match strategy {
      Strategy::PackageVersion(version) => {
        if let Some(ref cur_version) = self.package_version_with_cache(path) {
          if cur_version == version {
            ValidateResult::NoChanged
          } else {
            ValidateResult::Modified
          }
        } else {
          ValidateResult::Deleted
        }
      }
      Strategy::CompileTime(compile_time) => {
        if let Some(ref modified_time) = self.modified_time(path) {
          if modified_time > compile_time {
            ValidateResult::Modified
          } else {
            ValidateResult::NoChanged
          }
        } else {
          ValidateResult::Deleted
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{path::Path, sync::Arc};

  use rspack_fs::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};

  use super::{Strategy, StrategyHelper, ValidateResult};

  #[tokio::test]
  async fn should_strategy_works() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/packages/p1".into()).await.unwrap();
    fs.create_dir_all("/packages/p2".into()).await.unwrap();
    fs.write(
      "/packages/p1/package.json".into(),
      r#"{"version": "1.0.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write(
      "/packages/p2/package.json".into(),
      r#"{"version": "1.1.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/file1".into(), "abc".as_bytes()).await.unwrap();

    // compile_time
    let Strategy::CompileTime(time1) = StrategyHelper::compile_time() else {
      unreachable!()
    };
    std::thread::sleep(std::time::Duration::from_millis(100));
    let Strategy::CompileTime(time2) = StrategyHelper::compile_time() else {
      unreachable!()
    };
    assert!(time1 < time2);

    let mut helper = StrategyHelper::new(fs.clone());
    // modified_time
    assert_eq!(
      helper.modified_time(Path::new("/file1")),
      Some(fs.metadata("/file1".into()).unwrap().mtime_ms)
    );
    assert!(helper.modified_time(Path::new("/file2")).is_none());

    // package_version_with_cache
    assert_eq!(
      helper
        .package_version_with_cache(Path::new("/packages/p1/file"))
        .unwrap(),
      "1.0.0"
    );
    assert_eq!(
      helper
        .package_version_with_cache(Path::new("/packages/p2/file"))
        .unwrap(),
      "1.1.0"
    );
    assert_eq!(
      helper
        .package_version_with_cache(Path::new("/packages/p2/dir1/dir2/dir3/file"))
        .unwrap(),
      "1.1.0"
    );
    assert!(helper
      .package_version_with_cache(Path::new("/file1"))
      .is_none());
    assert!(helper
      .package_version_with_cache(Path::new("/file2"))
      .is_none());

    // package_version
    assert_eq!(
      helper
        .package_version(Path::new("/packages/p1/file"))
        .unwrap(),
      Strategy::PackageVersion("1.0.0".into())
    );
    assert_eq!(
      helper
        .package_version(Path::new("/packages/p2/file"))
        .unwrap(),
      Strategy::PackageVersion("1.1.0".into())
    );
    assert_eq!(
      helper
        .package_version(Path::new("/packages/p2/dir1/dir2/dir3/file"))
        .unwrap(),
      Strategy::PackageVersion("1.1.0".into())
    );
    assert!(helper.package_version(Path::new("/file1")).is_none());
    assert!(helper.package_version(Path::new("/file2")).is_none());

    // validate
    let now = StrategyHelper::compile_time();
    assert!(matches!(
      helper.validate(Path::new("/file1"), &now),
      ValidateResult::NoChanged
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1".into(), "abcd".as_bytes()).await.unwrap();
    assert!(matches!(
      helper.validate(Path::new("/file1"), &now),
      ValidateResult::Modified
    ));
    assert!(matches!(
      helper.validate(Path::new("/file2"), &now),
      ValidateResult::Deleted
    ));

    let version = Strategy::PackageVersion("1.0.0".into());
    assert!(matches!(
      helper.validate(Path::new("/packages/p1/file1"), &version),
      ValidateResult::NoChanged
    ));
    assert!(matches!(
      helper.validate(Path::new("/packages/p2/file1"), &version),
      ValidateResult::Modified
    ));
    assert!(matches!(
      helper.validate(Path::new("/file2"), &version),
      ValidateResult::Deleted
    ));
  }
}

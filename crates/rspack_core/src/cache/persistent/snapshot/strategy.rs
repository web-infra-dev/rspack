use std::{
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use dashmap::DashMap;
use rspack_cacheable::cacheable;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathDashMap, AssertUtf8};
use rustc_hash::FxHasher;

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
  /// This strategy will compare the compile time and the file update time.
  CompileTime(u64),

  /// Check by file hash
  ///
  /// This strategy will first compare the compile time and the file update time,
  /// and then compare the file hash if the file has been updated.
  PathHash { compile_time: u64, hash: u64 },
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
  fs: Arc<dyn ReadableFileSystem>,
  package_version_cache: ArcPathDashMap<Option<String>>,
  compile_time: u64,
}

impl StrategyHelper {
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("get current time failed")
      .as_millis() as u64;
    Self {
      fs,
      package_version_cache: Default::default(),
      compile_time: now,
    }
  }

  /// get path file modified time
  async fn modified_time(&self, path: &Path) -> Option<u64> {
    if let Ok(info) = self.fs.metadata(path.assert_utf8()).await {
      // return the larger of ctime and mtime
      if info.ctime_ms > info.mtime_ms {
        Some(info.ctime_ms)
      } else {
        Some(info.mtime_ms)
      }
    } else {
      None
    }
  }

  /// get path file version in package.json
  #[async_recursion::async_recursion]
  async fn package_version_with_cache(&self, path: &ArcPath) -> Option<String> {
    if let Some(version) = self.package_version_cache.get(path) {
      return version.clone();
    }

    let mut res = None;
    if let Ok(content) = self.fs.read(&path.join("package.json").assert_utf8()).await
      && let Ok(mut package_json) =
        serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&content)
      && let Some(serde_json::Value::String(version)) = package_json.remove("version")
    {
      res = Some(version);
    }

    if res.is_none()
      && let Some(p) = path.parent()
    {
      res = self.package_version_with_cache(&ArcPath::from(p)).await;
    }

    self.package_version_cache.insert(path.into(), res.clone());
    res
  }

  /// get path file content hash
  async fn content_hash(&self, path: &Path) -> Option<u64> {
    // currently only supports files
    // TODO add cache if directory hash is supported
    let Ok(content) = self.fs.read(path.assert_utf8()).await else {
      return None;
    };
    let mut hasher = FxHasher::default();
    content.hash(&mut hasher);
    Some(hasher.finish())
  }

  /// get current time as compile time strategy
  pub fn compile_time(&self) -> Strategy {
    Strategy::CompileTime(self.compile_time)
  }
  /// get path file package version strategy
  pub async fn package_version(&self, path: &ArcPath) -> Option<Strategy> {
    self
      .package_version_with_cache(path)
      .await
      .map(Strategy::PackageVersion)
  }
  /// get path file hash strategy
  pub async fn path_hash(&self, path: &Path) -> Option<Strategy> {
    let hash = self.content_hash(path).await?;
    Some(Strategy::PathHash {
      compile_time: self.compile_time,
      hash,
    })
  }

  /// validate path file by target strategy
  pub async fn validate(&self, path: &ArcPath, strategy: &Strategy) -> ValidateResult {
    let Some(modified_time) = self.modified_time(path).await else {
      return ValidateResult::Deleted;
    };
    match strategy {
      Strategy::PackageVersion(version) => {
        let Some(ref cur_version) = self.package_version_with_cache(path).await else {
          return ValidateResult::Deleted;
        };
        if cur_version == version {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::CompileTime(compile_time) => {
        if &modified_time > compile_time {
          ValidateResult::Modified
        } else {
          ValidateResult::NoChanged
        }
      }
      Strategy::PathHash { compile_time, hash } => {
        if &modified_time < compile_time {
          return ValidateResult::NoChanged;
        }
        let Some(file_hash) = self.content_hash(path).await else {
          return ValidateResult::Deleted;
        };
        if &file_hash == hash {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::ArcPath;

  use super::{Strategy, StrategyHelper, ValidateResult};

  #[tokio::test]
  async fn compile_time() {
    let fs = Arc::new(MemoryFileSystem::default());
    let helper = StrategyHelper::new(fs.clone());
    let compile_time_1 = helper.compile_time();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let compile_time_2 = helper.compile_time();
    assert_eq!(compile_time_1, compile_time_2);
  }

  #[tokio::test]
  async fn package_version() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/packages/p1".into()).await.unwrap();
    fs.write(
      "/packages/p1/package.json".into(),
      r#"{"version": "1.2.0"}"#.as_bytes(),
    )
    .await
    .unwrap();

    let helper = StrategyHelper::new(fs.clone());
    assert_eq!(
      helper
        .package_version(&ArcPath::from("/packages/p1/file.js"))
        .await,
      Some(Strategy::PackageVersion("1.2.0".into()))
    );
    assert_eq!(
      helper
        .package_version(&ArcPath::from("/packages/p2/file.js"))
        .await,
      None
    );
  }

  #[tokio::test]
  async fn path_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();

    let helper = StrategyHelper::new(fs.clone());
    assert_eq!(
      helper.path_hash(&ArcPath::from("/not_exist.js")).await,
      None
    );

    let hash1 = helper.path_hash(&ArcPath::from("/hash.js")).await;
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();
    let hash2 = helper.path_hash(&ArcPath::from("/hash.js")).await;
    assert_eq!(hash1, hash2);

    fs.write("/hash.js".into(), "abcd".as_bytes())
      .await
      .unwrap();
    let hash3 = helper.path_hash(&ArcPath::from("/hash.js")).await;
    assert_ne!(hash1, hash3);
  }

  #[tokio::test]
  async fn validate_compile_time() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/file1.js".into(), "abc".as_bytes())
      .await
      .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    let helper = StrategyHelper::new(fs.clone());
    let strategy = helper.compile_time();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::NoChanged
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1.js".into(), "abc".as_bytes())
      .await
      .unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::Modified
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.remove_file("/file1.js".into()).await.unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::Deleted
    ));
  }

  #[tokio::test]
  async fn validate_package_version() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/packages/lib".into()).await.unwrap();
    fs.write(
      "/packages/lib/package.json".into(),
      r#"{"version": "1.0.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/packages/lib/file.js".into(), "abc".as_bytes())
      .await
      .unwrap();

    let strategy = Strategy::PackageVersion("1.0.0".into());
    let helper = StrategyHelper::new(fs.clone());
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/packages/lib/file.js"), &strategy)
        .await,
      ValidateResult::NoChanged
    ));

    helper.package_version_cache.clear();
    fs.write(
      "/packages/lib/package.json".into(),
      r#"{"version": "1.2.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/packages/lib/file.js"), &strategy)
        .await,
      ValidateResult::Modified
    ));

    helper.package_version_cache.clear();
    fs.remove_file("/packages/lib/package.json".into())
      .await
      .unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/packages/lib/file.js"), &strategy)
        .await,
      ValidateResult::Deleted
    ));
  }

  #[tokio::test]
  async fn validate_path_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/file1.js".into(), "abc".as_bytes())
      .await
      .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    let helper = StrategyHelper::new(fs.clone());
    let strategy = helper.path_hash(&ArcPath::from("/file1.js")).await.unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::NoChanged
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1.js".into(), "abc".as_bytes())
      .await
      .unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::NoChanged
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1.js".into(), "abcd".as_bytes())
      .await
      .unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::Modified
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.remove_file("/file1.js".into()).await.unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::Deleted
    ));
  }
}

mod hash_helper;
mod package_helper;

use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, AssertUtf8};

use self::{
  hash_helper::{ContentHash, HashHelper, TimestampHash},
  package_helper::PackageHelper,
};
use super::{SnapshotOptions, SnapshotStrategyOptions};

/// Snapshot check strategy
#[cacheable]
#[derive(Debug)]
pub enum Strategy {
  /// Check by package version
  ///
  /// This strategy will find the package.json in the parent directory, and
  /// compares the version field.
  PackageVersion(String),

  /// Check by file hash
  ///
  /// This strategy will compare the file hash.
  FileHash { hash: u64 },

  /// Check by file timestamp
  FileTimestamp { mtime: u64 },

  /// Check by file timestamp and hash
  ///
  /// This strategy will first compare the modified time,
  /// and then compare the file hash when the modified time changed.
  FileTimestampAndHash { mtime: u64, hash: u64 },

  /// Check by dir hash
  ///
  /// This strategy will compare the content hash of all files within the directory.
  DirHash { hash: u64 },

  /// Check by dir timestamp hash
  DirTimestamp { timestamp_hash: u64 },

  /// Check by dir timestamp hash and content hash
  DirTimestampAndHash { timestamp_hash: u64, hash: u64 },

  /// Check missing file
  ///
  /// This strategy indicates that the current file is in a missing state,
  /// and will return ValidateResult::Modified if it exists.
  Missing,

  /// Check failed snapshot
  ///
  /// This strategy represents a snapshot that could not be created or
  /// validated correctly and should be treated as invalid.
  Failed,
}

impl PartialEq for Strategy {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::PackageVersion(v1), Self::PackageVersion(v2)) => v1 == v2,
      (Self::FileHash { hash: h1, .. }, Self::FileHash { hash: h2, .. }) => h1 == h2,
      (Self::FileTimestamp { mtime: m1 }, Self::FileTimestamp { mtime: m2 }) => m1 == m2,
      (
        Self::FileTimestampAndHash { hash: h1, .. },
        Self::FileTimestampAndHash { hash: h2, .. },
      ) => h1 == h2,
      (Self::DirHash { hash: h1, .. }, Self::DirHash { hash: h2, .. }) => h1 == h2,
      (Self::DirTimestamp { timestamp_hash: h1 }, Self::DirTimestamp { timestamp_hash: h2 }) => {
        h1 == h2
      }
      (Self::DirTimestampAndHash { hash: h1, .. }, Self::DirTimestampAndHash { hash: h2, .. }) => {
        h1 == h2
      }
      (Self::Missing, Self::Missing) => true,
      (Self::Failed, Self::Failed) => true,
      _ => false,
    }
  }
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
  package_helper: Arc<PackageHelper>,
  hash_helper: HashHelper,
}

impl StrategyHelper {
  pub fn new(fs: Arc<dyn ReadableFileSystem>, snapshot_options: Arc<SnapshotOptions>) -> Self {
    let package_helper = Arc::new(PackageHelper::new(fs.clone()));
    Self {
      fs: fs.clone(),
      hash_helper: HashHelper::new(fs, snapshot_options, package_helper.clone()),
      package_helper,
    }
  }

  /// get path file modified time
  async fn modified_time(&self, path: &ArcPath) -> Option<u64> {
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

  /// get path file package version strategy
  pub async fn package_version(&self, path: &ArcPath) -> Option<Strategy> {
    self
      .package_helper
      .package_version(path)
      .await
      .map(Strategy::PackageVersion)
  }

  /// get path file hash strategy
  pub async fn file_hash(&self, path: &ArcPath) -> Strategy {
    if let Some(ContentHash { hash, mtime }) = self.hash_helper.file_hash(path).await {
      Strategy::FileTimestampAndHash { mtime, hash }
    } else {
      Strategy::Missing
    }
  }

  /// get path file strategy
  pub async fn file_strategy(
    &self,
    path: &ArcPath,
    strategy_options: SnapshotStrategyOptions,
  ) -> Strategy {
    match (strategy_options.hash, strategy_options.timestamp) {
      (true, true) => self.file_hash(path).await,
      (true, false) => {
        if let Some(ContentHash { hash, .. }) = self.hash_helper.file_hash(path).await {
          Strategy::FileHash { hash }
        } else {
          Strategy::Missing
        }
      }
      (false, true) => {
        if let Some(mtime) = self.modified_time(path).await {
          Strategy::FileTimestamp { mtime }
        } else {
          Strategy::Missing
        }
      }
      (false, false) => Strategy::Failed,
    }
  }

  /// get path context hash strategy
  pub async fn dir_hash(&self, path: &ArcPath) -> Strategy {
    if let Some(ContentHash { hash, .. }) = self.hash_helper.dir_hash(path).await {
      Strategy::DirHash { hash }
    } else {
      Strategy::Failed
    }
  }

  /// get path context strategy
  pub async fn dir_strategy(
    &self,
    path: &ArcPath,
    strategy_options: SnapshotStrategyOptions,
  ) -> Strategy {
    match (strategy_options.hash, strategy_options.timestamp) {
      (true, true) => {
        let Some(TimestampHash {
          hash: timestamp_hash,
          ..
        }) = self.hash_helper.dir_timestamp_hash(path).await
        else {
          return Strategy::Failed;
        };
        if let Some(ContentHash { hash, .. }) = self.hash_helper.dir_hash(path).await {
          Strategy::DirTimestampAndHash {
            timestamp_hash,
            hash,
          }
        } else {
          Strategy::Failed
        }
      }
      (true, false) => self.dir_hash(path).await,
      (false, true) => {
        if let Some(TimestampHash {
          hash: timestamp_hash,
          ..
        }) = self.hash_helper.dir_timestamp_hash(path).await
        {
          Strategy::DirTimestamp { timestamp_hash }
        } else {
          Strategy::Failed
        }
      }
      (false, false) => Strategy::Failed,
    }
  }

  /// validate path file by target strategy
  pub async fn validate(&self, path: &ArcPath, strategy: &Strategy) -> ValidateResult {
    match strategy {
      Strategy::PackageVersion(version) => {
        let Some(ref cur_version) = self.package_helper.package_version(path).await else {
          return ValidateResult::Deleted;
        };
        if cur_version == version {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::FileHash { hash } => {
        let Some(ContentHash { hash: cur_hash, .. }) = self.hash_helper.file_hash(path).await
        else {
          return ValidateResult::Deleted;
        };
        if &cur_hash == hash {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::FileTimestamp { mtime } => {
        let Some(modified_time) = self.modified_time(path).await else {
          return ValidateResult::Deleted;
        };
        if &modified_time == mtime {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::FileTimestampAndHash { mtime, hash } => {
        let Some(modified_time) = self.modified_time(path).await else {
          return ValidateResult::Deleted;
        };
        if &modified_time == mtime {
          return ValidateResult::NoChanged;
        }
        let Some(ContentHash { hash: cur_hash, .. }) = self.hash_helper.file_hash(path).await
        else {
          return ValidateResult::Deleted;
        };
        if &cur_hash == hash {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::DirHash { hash } => {
        let Some(ContentHash { hash: cur_hash, .. }) = self.hash_helper.dir_hash(path).await else {
          return ValidateResult::Deleted;
        };
        if &cur_hash == hash {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::DirTimestamp { timestamp_hash } => {
        let Some(TimestampHash {
          hash: cur_timestamp_hash,
          ..
        }) = self.hash_helper.dir_timestamp_hash(path).await
        else {
          return ValidateResult::Deleted;
        };
        if &cur_timestamp_hash == timestamp_hash {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::DirTimestampAndHash {
        timestamp_hash,
        hash,
      } => {
        let Some(TimestampHash {
          hash: cur_timestamp_hash,
          ..
        }) = self.hash_helper.dir_timestamp_hash(path).await
        else {
          return ValidateResult::Deleted;
        };
        if &cur_timestamp_hash == timestamp_hash {
          return ValidateResult::NoChanged;
        }
        let Some(ContentHash { hash: cur_hash, .. }) = self.hash_helper.dir_hash(path).await else {
          return ValidateResult::Deleted;
        };
        if &cur_hash == hash {
          ValidateResult::NoChanged
        } else {
          ValidateResult::Modified
        }
      }
      Strategy::Missing => {
        if self.modified_time(path).await.is_some() {
          ValidateResult::Modified
        } else {
          ValidateResult::NoChanged
        }
      }
      Strategy::Failed => ValidateResult::Modified,
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
    let helper = StrategyHelper::new(fs.clone(), Default::default());
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/packages/lib/file.js"), &strategy)
        .await,
      ValidateResult::NoChanged
    ));

    let helper = StrategyHelper::new(fs.clone(), Default::default());
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

    let helper = StrategyHelper::new(fs.clone(), Default::default());
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
  async fn validate_file_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/file1.js".into(), "abc".as_bytes())
      .await
      .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    let helper = StrategyHelper::new(fs.clone(), Default::default());
    let strategy = helper.file_hash(&ArcPath::from("/file1.js")).await;
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::NoChanged
    ));

    std::thread::sleep(std::time::Duration::from_millis(100));
    let helper = StrategyHelper::new(fs.clone(), Default::default());
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
    let helper = StrategyHelper::new(fs.clone(), Default::default());
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
    let helper = StrategyHelper::new(fs.clone(), Default::default());
    fs.remove_file("/file1.js".into()).await.unwrap();
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::Deleted
    ));
  }

  #[tokio::test]
  async fn validate_missing() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/file1.js".into(), "abc".as_bytes())
      .await
      .unwrap();

    let helper = StrategyHelper::new(fs.clone(), Default::default());
    let strategy = Strategy::Missing;
    assert!(matches!(
      helper
        .validate(&ArcPath::from("/file1.js"), &strategy)
        .await,
      ValidateResult::Modified
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
      ValidateResult::NoChanged
    ));
  }
}

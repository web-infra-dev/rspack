mod option;
mod strategy;

use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathSet};

use self::strategy::{StrategyHelper, ValidateResult};
pub use self::{
  option::{PathMatcher, SnapshotOptions},
  strategy::Strategy,
};
use super::{codec::CacheCodec, storage::Storage};
use crate::FutureConsumer;

pub const SCOPE: &str = "snapshot";

/// Represents the type of dependency snapshot being captured.
#[derive(Debug, Clone, Copy)]
pub enum SnapshotType {
  /// for compilation.file_dependencies
  FILE,
  /// for compilation.context_dependencies
  CONTEXT,
  /// for compilation.missing_dependencies
  MISSING,
  /// for compilation.build_dependencies
  BUILD,
}

/// Snapshot is used to check if files have been modified or deleted.
///
/// Snapshot will generate `Strategy` for target file, and check the modification
/// through the generated `Strategy`
#[derive(Debug)]
pub struct Snapshot {
  scope: &'static str,
  options: Arc<SnapshotOptions>,
  fs: Arc<dyn ReadableFileSystem>,
  storage: Arc<dyn Storage>,
  codec: Arc<CacheCodec>,
}

impl Snapshot {
  pub fn new(
    options: SnapshotOptions,
    fs: Arc<dyn ReadableFileSystem>,
    storage: Arc<dyn Storage>,
    codec: Arc<CacheCodec>,
  ) -> Self {
    Self {
      scope: SCOPE,
      options: Arc::new(options),
      fs,
      storage,
      codec,
    }
  }

  pub fn new_with_scope(
    scope: &'static str,
    options: SnapshotOptions,
    fs: Arc<dyn ReadableFileSystem>,
    storage: Arc<dyn Storage>,
    codec: Arc<CacheCodec>,
  ) -> Self {
    Self {
      scope,
      options: Arc::new(options),
      fs,
      storage,
      codec,
    }
  }

  async fn calc_strategy(
    options: &Arc<SnapshotOptions>,
    helper: &Arc<StrategyHelper>,
    path: &ArcPath,
    snapshot_type: SnapshotType,
  ) -> Option<Strategy> {
    let path_str = path.to_string_lossy();
    if options.is_immutable_path(&path_str) {
      return None;
    }
    if options.is_managed_path(&path_str)
      && let Some(v) = helper.package_version(path).await
    {
      return Some(v);
    }
    Some(match snapshot_type {
      SnapshotType::FILE => helper.file_hash(path).await,
      SnapshotType::MISSING => Strategy::Missing,
      SnapshotType::CONTEXT | SnapshotType::BUILD => helper.dir_hash(path).await,
    })
  }

  #[tracing::instrument("Cache::Snapshot::add", skip_all)]
  pub async fn add(&self, paths: impl Iterator<Item = ArcPath>, snapshot_type: SnapshotType) {
    let helper = Arc::new(StrategyHelper::new(self.fs.clone()));
    let codec = self.codec.clone();
    // TODO merge package version file
    paths
      .map(|path| {
        let helper = helper.clone();
        let options = self.options.clone();
        let codec = codec.clone();
        async move {
          let strategy = Self::calc_strategy(&options, &helper, &path, snapshot_type).await?;
          Some((
            codec.encode(&path).expect("should encode success"),
            codec.encode(&strategy).expect("should encode success"),
          ))
        }
      })
      .fut_consume(|data| {
        if let Some((key, value)) = data {
          self.storage.set(self.scope, key, value);
        }
      })
      .await;
  }

  pub fn remove(&self, paths: impl Iterator<Item = ArcPath>) {
    for item in paths {
      self
        .storage
        .remove(self.scope, item.as_os_str().as_encoded_bytes())
    }
  }

  #[allow(clippy::type_complexity)]
  #[tracing::instrument("Cache::Snapshot::calc_modified_path", skip_all)]
  pub async fn calc_modified_paths(&self) -> Result<(bool, ArcPathSet, ArcPathSet, ArcPathSet)> {
    let mut modified_path = ArcPathSet::default();
    let mut deleted_path = ArcPathSet::default();
    let mut no_change_path = ArcPathSet::default();
    let helper = Arc::new(StrategyHelper::new(self.fs.clone()));
    let codec = self.codec.clone();

    let data = self.storage.load(self.scope).await?;
    let is_hot_start = !data.is_empty();
    data
      .into_iter()
      .map(|(key, value)| {
        let helper = helper.clone();
        let codec = codec.clone();
        async move {
          let path: ArcPath = codec.decode(&key).expect("should decode success");
          let strategy: Strategy = codec.decode(&value).expect("should decode success");
          let validate = helper.validate(&path, &strategy).await;
          (path, validate)
        }
      })
      .fut_consume(|(path, validate)| match validate {
        ValidateResult::Modified => {
          modified_path.insert(path);
        }
        ValidateResult::Deleted => {
          deleted_path.insert(path);
        }
        ValidateResult::NoChanged => {
          no_change_path.insert(path);
        }
      })
      .await;

    Ok((is_hot_start, modified_path, deleted_path, no_change_path))
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::ArcPath;

  use super::{
    super::{codec::CacheCodec, storage::MemoryStorage},
    PathMatcher, Snapshot, SnapshotOptions, SnapshotType,
  };

  macro_rules! p {
    ($tt:tt) => {
      ArcPath::from(std::path::Path::new($tt))
    };
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn should_snapshot_work() {
    let fs = Arc::new(MemoryFileSystem::default());
    let storage = Arc::new(MemoryStorage::default());
    let codec = Arc::new(CacheCodec::new(None));
    let options = SnapshotOptions::new(
      vec![PathMatcher::String("constant".into())],
      vec![PathMatcher::String("node_modules/project".into())],
      vec![PathMatcher::String("node_modules".into())],
    );

    fs.create_dir_all("/node_modules/project".into())
      .await
      .unwrap();
    fs.create_dir_all("/node_modules/lib".into()).await.unwrap();
    fs.write("/file1".into(), "abc".as_bytes()).await.unwrap();
    fs.write("/constant".into(), "abc".as_bytes())
      .await
      .unwrap();
    fs.write(
      "/node_modules/project/package.json".into(),
      r#"{"version":"1.0.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/node_modules/project/file1".into(), "abc".as_bytes())
      .await
      .unwrap();
    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"version":"1.1.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/node_modules/lib/file1".into(), "abc".as_bytes())
      .await
      .unwrap();

    let snapshot = Snapshot::new(options, fs.clone(), storage, codec);

    snapshot
      .add(
        [
          p!("/file1"),
          p!("/constant"),
          p!("/node_modules/project/file1"),
          p!("/node_modules/lib/file1"),
        ]
        .into_iter(),
        SnapshotType::FILE,
      )
      .await;
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1".into(), "abcd".as_bytes()).await.unwrap();
    fs.write("/constant".into(), "abcd".as_bytes())
      .await
      .unwrap();
    fs.write("/node_modules/project/file1".into(), "abcd".as_bytes())
      .await
      .unwrap();
    fs.write("/node_modules/lib/file1".into(), "abcd".as_bytes())
      .await
      .unwrap();

    let (is_hot_start, modified_paths, deleted_paths, no_change_paths) =
      snapshot.calc_modified_paths().await.unwrap();
    assert!(is_hot_start);
    assert!(deleted_paths.is_empty());
    assert!(!modified_paths.contains(&p!("/constant")));
    assert!(modified_paths.contains(&p!("/file1")));
    assert!(modified_paths.contains(&p!("/node_modules/project/file1")));
    assert!(!modified_paths.contains(&p!("/node_modules/lib/file1")));
    assert_eq!(no_change_paths.len(), 1);

    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"version":"1.3.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    snapshot
      .add([p!("/file1")].into_iter(), SnapshotType::FILE)
      .await;
    let (is_hot_start, modified_paths, deleted_paths, no_change_paths) =
      snapshot.calc_modified_paths().await.unwrap();
    assert!(is_hot_start);
    assert!(deleted_paths.is_empty());
    assert!(!modified_paths.contains(&p!("/constant")));
    assert!(!modified_paths.contains(&p!("/file1")));
    assert!(modified_paths.contains(&p!("/node_modules/project/file1")));
    assert!(modified_paths.contains(&p!("/node_modules/lib/file1")));
    assert_eq!(no_change_paths.len(), 1);
  }
}

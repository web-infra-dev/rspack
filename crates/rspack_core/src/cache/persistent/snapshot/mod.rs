mod option;
mod strategy;

use std::{path::Path, sync::Arc};

use rspack_cacheable::{from_bytes, to_bytes};
use rspack_error::Result;
use rspack_fs::FileSystem;
use rspack_paths::{ArcPath, AssertUtf8};
use rustc_hash::FxHashSet as HashSet;

pub use self::option::{PathMatcher, SnapshotOptions};
use self::strategy::{Strategy, StrategyHelper, ValidateResult};
use super::storage::Storage;

const SCOPE: &str = "snapshot";

/// Snapshot is used to check if files have been modified or deleted.
///
/// Snapshot will generate `Strategy` for target file, and check the modification
/// through the generated `Strategy`
#[derive(Debug)]
pub struct Snapshot {
  options: SnapshotOptions,
  fs: Arc<dyn FileSystem>,
  storage: Arc<dyn Storage>,
}

impl Snapshot {
  pub fn new(options: SnapshotOptions, fs: Arc<dyn FileSystem>, storage: Arc<dyn Storage>) -> Self {
    Self {
      options,
      fs,
      storage,
    }
  }

  pub async fn add(&self, paths: impl Iterator<Item = &Path>) {
    let default_strategy = StrategyHelper::compile_time();
    let mut helper = StrategyHelper::new(self.fs.clone());
    // TODO use multi thread
    // TODO merge package version file
    for path in paths {
      let utf8_path = path.assert_utf8();
      // check path exists
      if self.fs.metadata(utf8_path).is_err() {
        continue;
      }
      // TODO directory path should check all sub file
      let path_str = utf8_path.as_str();
      if self.options.is_immutable_path(path_str) {
        continue;
      }
      if self.options.is_managed_path(path_str) {
        if let Some(v) = helper.package_version(path).await {
          self.storage.set(
            SCOPE,
            path.as_os_str().as_encoded_bytes().to_vec(),
            to_bytes::<_, ()>(&v, &()).expect("should to bytes success"),
          );
          continue;
        }
      }
      // compiler time
      self.storage.set(
        SCOPE,
        path.as_os_str().as_encoded_bytes().to_vec(),
        to_bytes::<_, ()>(&default_strategy, &()).expect("should to bytes success"),
      );
    }
  }

  pub fn remove(&self, paths: impl Iterator<Item = &Path>) {
    for item in paths {
      self
        .storage
        .remove(SCOPE, item.as_os_str().as_encoded_bytes())
    }
  }

  pub async fn calc_modified_paths(&self) -> Result<(HashSet<ArcPath>, HashSet<ArcPath>)> {
    let mut helper = StrategyHelper::new(self.fs.clone());
    let mut modified_path = HashSet::default();
    let mut deleted_path = HashSet::default();

    // TODO use multi thread
    for (key, value) in self.storage.load(SCOPE).await? {
      let path: ArcPath = Path::new(&*String::from_utf8_lossy(&key)).into();
      let strategy: Strategy =
        from_bytes::<Strategy, ()>(&value, &()).expect("should from bytes success");
      match helper.validate(&path, &strategy).await {
        ValidateResult::Modified => {
          modified_path.insert(path);
        }
        ValidateResult::Deleted => {
          deleted_path.insert(path);
        }
        ValidateResult::NoChanged => {}
      }
    }
    Ok((modified_path, deleted_path))
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};

  use super::super::storage::MemoryStorage;
  use super::{PathMatcher, Snapshot, SnapshotOptions};

  macro_rules! p {
    ($tt:tt) => {
      std::path::Path::new($tt)
    };
  }

  #[tokio::test]
  async fn should_snapshot_work() {
    let fs = Arc::new(MemoryFileSystem::default());
    let storage = Arc::new(MemoryStorage::default());
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

    let snapshot = Snapshot::new(options, fs.clone(), storage);

    snapshot
      .add(
        [
          p!("/file1"),
          p!("/constant"),
          p!("/node_modules/project/file1"),
          p!("/node_modules/lib/file1"),
        ]
        .into_iter(),
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

    let (modified_paths, deleted_paths) = snapshot.calc_modified_paths().await.unwrap();
    assert!(deleted_paths.is_empty());
    assert!(!modified_paths.contains(p!("/constant")));
    assert!(modified_paths.contains(p!("/file1")));
    assert!(modified_paths.contains(p!("/node_modules/project/file1")));
    assert!(!modified_paths.contains(p!("/node_modules/lib/file1")));

    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"version":"1.3.0"}"#.as_bytes(),
    )
    .await
    .unwrap();
    snapshot.add([p!("/file1")].into_iter()).await;
    let (modified_paths, deleted_paths) = snapshot.calc_modified_paths().await.unwrap();
    assert!(deleted_paths.is_empty());
    assert!(!modified_paths.contains(p!("/constant")));
    assert!(!modified_paths.contains(p!("/file1")));
    assert!(modified_paths.contains(p!("/node_modules/project/file1")));
    assert!(modified_paths.contains(p!("/node_modules/lib/file1")));
  }
}

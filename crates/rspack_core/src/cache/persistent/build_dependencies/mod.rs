mod helper;

use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathSet, AssertUtf8};
use rustc_hash::FxHashSet as HashSet;

use self::helper::{Helper, is_node_package_path};
use super::{
  snapshot::{Snapshot, SnapshotOptions},
  storage::Storage,
};

const SCOPE: &str = "build_dependencies";

pub type BuildDepsOptions = Vec<PathBuf>;

/// Build dependencies manager
#[derive(Debug)]
pub struct BuildDeps {
  /// The build dependencies has been added to snapshot.
  ///
  /// This field is used to avoid adding duplicate build dependencies to the snapshot.
  added: ArcPathSet,
  /// The pending dependencies.
  ///
  /// The next time the add method is called, this path will be additionally added.
  pending: ArcPathSet,
  /// The snapshot which is used to save build dependencies.
  snapshot: Snapshot,
  storage: Arc<dyn Storage>,
  fs: Arc<dyn ReadableFileSystem>,
}

impl BuildDeps {
  pub fn new(
    options: &BuildDepsOptions,
    snapshot_options: &SnapshotOptions,
    fs: Arc<dyn ReadableFileSystem>,
    storage: Arc<dyn Storage>,
  ) -> Self {
    Self {
      added: Default::default(),
      pending: options.iter().map(|v| ArcPath::from(v.as_path())).collect(),
      snapshot: Snapshot::new_with_scope(
        SCOPE,
        snapshot_options.clone(),
        fs.clone(),
        storage.clone(),
      ),
      storage,
      fs,
    }
  }

  /// Add build dependencies
  ///
  /// For performance reasons, recursive searches will stop for build dependencies in node_modules.
  pub async fn add(&mut self, data: impl Iterator<Item = ArcPath>) -> Vec<String> {
    let mut helper = Helper::new(self.fs.clone());
    let mut new_deps = HashSet::default();
    let mut queue = VecDeque::new();
    queue.extend(std::mem::take(&mut self.pending));
    queue.extend(data);
    loop {
      let Some(current) = queue.pop_front() else {
        break;
      };
      if !self.added.insert(current.clone()) {
        continue;
      }
      new_deps.insert(current.clone());
      if is_node_package_path(&current) {
        // node package path skip recursive search.
        continue;
      }
      if let Some(children) = helper.resolve(current.assert_utf8()).await {
        queue.extend(children.iter().map(|item| item.as_path().into()));
      }
    }

    self.snapshot.add(new_deps.into_iter()).await;
    helper.into_warnings()
  }

  /// Validate build dependencies
  ///
  /// If any build dependencies have changed, this method will reset storage.
  pub async fn validate(&mut self) -> Result<()> {
    let (_, modified_files, removed_files, no_changed_files) =
      self.snapshot.calc_modified_paths().await?;

    if !modified_files.is_empty() || !removed_files.is_empty() {
      self.storage.reset().await;

      tracing::info!(
        "BuildDependencies: cache invalidate by modified_files {modified_files:?} and removed_files {removed_files:?}"
      );
      return Ok(());
    }
    self.added = no_changed_files;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::{path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_storage::Storage;

  use super::{super::storage::MemoryStorage, BuildDeps, SCOPE, SnapshotOptions};
  #[tokio::test]
  async fn build_dependencies_test() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/configs/test".into()).await.unwrap();
    fs.write("/configs/a.js".into(), r#"console.log('a')"#.as_bytes())
      .await
      .unwrap();
    fs.write(
      "/configs/test/b.js".into(),
      r#"console.log('b')"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write(
      "/configs/test/b1.js".into(),
      r#"console.log('b1')"#.as_bytes(),
    )
    .await
    .unwrap();
    fs.write("/configs/c.txt".into(), r#"123"#.as_bytes())
      .await
      .unwrap();
    fs.write("/a.js".into(), r#"require("./b")"#.as_bytes())
      .await
      .unwrap();
    fs.write("/b.js".into(), r#"require("./c"); console.log("#.as_bytes())
      .await
      .unwrap();
    fs.write("/c.js".into(), r#"console.log('c')"#.as_bytes())
      .await
      .unwrap();
    fs.write("/index.js".into(), r#"import "./a""#.as_bytes())
      .await
      .unwrap();

    let options = vec![PathBuf::from("/index.js"), PathBuf::from("/configs")];
    let snapshot_options = SnapshotOptions::default();
    let storage = Arc::new(MemoryStorage::default());
    let mut build_deps = BuildDeps::new(&options, &snapshot_options, fs.clone(), storage.clone());
    let warnings = build_deps.add(vec![].into_iter()).await;
    assert_eq!(warnings.len(), 1);
    let data = storage.load(SCOPE).await.expect("should load success");
    assert_eq!(data.len(), 9);

    let mut build_deps = BuildDeps::new(&options, &snapshot_options, fs.clone(), storage.clone());
    fs.write("/b.js".into(), r#"require("./c")"#.as_bytes())
      .await
      .unwrap();
    build_deps
      .validate()
      .await
      .expect("should validate success");

    let data = storage.load(SCOPE).await.expect("should load success");
    assert_eq!(data.len(), 0);
    let warnings = build_deps.add(vec![].into_iter()).await;
    assert_eq!(warnings.len(), 0);
    let data = storage.load(SCOPE).await.expect("should load success");
    assert_eq!(data.len(), 10);
  }
}

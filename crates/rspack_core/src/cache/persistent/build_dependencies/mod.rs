mod helper;

use std::{collections::VecDeque, future::Future, path::PathBuf, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathSet, AssertUtf8};
use rustc_hash::FxHashSet as HashSet;

use self::helper::{Helper, is_node_package_path};
use super::{
  snapshot::{Snapshot, SnapshotScope},
  storage::{Storage, StorageUpdates},
};
use crate::CompilationLogger;

pub const SCOPE: &str = "build_dependencies";

pub type BuildDepsOptions = Vec<PathBuf>;

#[derive(Debug)]
pub enum BuildDepsValidationResult {
  Valid {
    tracked_files: usize,
  },
  Invalid {
    modified_files: ArcPathSet,
    removed_files: ArcPathSet,
  },
}

/// Build dependencies manager
#[derive(Debug)]
pub struct BuildDeps {
  /// The build dependencies has been added to snapshot.
  ///
  /// This field is used to avoid adding duplicate build dependencies to the snapshot.
  added: ArcPathSet,
  /// The pending dependencies.
  ///
  /// The next update task will additionally add these paths.
  pending: ArcPathSet,
  /// The snapshot which is used to save build dependencies.
  snapshot: Arc<Snapshot>,
  fs: Arc<dyn ReadableFileSystem>,
}

impl BuildDeps {
  pub fn new(
    options: &BuildDepsOptions,
    fs: Arc<dyn ReadableFileSystem>,
    snapshot: Arc<Snapshot>,
  ) -> Self {
    Self {
      added: Default::default(),
      pending: options.iter().map(|v| ArcPath::from(v.as_path())).collect(),
      snapshot,
      fs,
    }
  }

  /// Reset build dependencies scope in storage
  pub fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SnapshotScope::BUILD.name());
  }

  /// Saves update changes for build dependencies.
  ///
  /// For performance reasons, recursive searches will stop for build dependencies in node_modules.
  pub fn save_updates_task(
    &mut self,
    data: impl IntoIterator<Item = ArcPath>,
    logger: CompilationLogger,
  ) -> impl Future<Output = StorageUpdates> + Send + 'static {
    let fs = self.fs.clone();
    let snapshot = self.snapshot.clone();
    let pending = std::mem::take(&mut self.pending);
    let data = data.into_iter().collect::<Vec<_>>();
    let mut queue = VecDeque::new();
    for item in pending.into_iter().chain(data) {
      if self.added.insert(item.clone()) {
        queue.push_back(item);
      }
    }

    async move {
      let mut helper = Helper::new(fs, logger);
      let mut new_deps = HashSet::default();
      while let Some(current) = queue.pop_front() {
        if !new_deps.insert(current.clone()) {
          continue;
        }
        if is_node_package_path(&current) {
          // node package path skip recursive search.
          continue;
        }
        if let Some(children) = helper.resolve(current.assert_utf8()).await {
          queue.extend(children.iter().map(|item| item.as_path().into()));
        }
      }

      snapshot
        .save_scope_updates(SnapshotScope::BUILD, new_deps.into_iter().collect(), vec![])
        .await
    }
  }

  /// Validate build dependencies
  ///
  /// If any build dependencies have changed, this method will return an invalid result.
  pub async fn validate(&mut self, storage: &dyn Storage) -> Result<BuildDepsValidationResult> {
    let (_, modified_files, removed_files, no_changed_files) = self
      .snapshot
      .calc_modified_paths(storage, SnapshotScope::BUILD)
      .await?;

    if !modified_files.is_empty() || !removed_files.is_empty() {
      return Ok(BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      });
    }
    let tracked_files = no_changed_files.len();
    self.added = no_changed_files;
    Ok(BuildDepsValidationResult::Valid { tracked_files })
  }
}

#[cfg(test)]
mod test {
  use std::{path::PathBuf, sync::Arc};

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};

  use super::{
    super::{
      codec::CacheCodec,
      snapshot::{Snapshot, SnapshotOptions, SnapshotScope},
      storage::{MemoryStorage, Storage, StorageUpdates},
    },
    BuildDeps, BuildDepsValidationResult,
  };
  use crate::{CompilationLogger, CompilationLogging, LogType};

  fn test_logger(name: &str) -> (CompilationLogger, CompilationLogging) {
    let logging = CompilationLogging::default();
    (
      CompilationLogger::new(name.to_string(), logging.clone()),
      logging,
    )
  }

  fn warn_count(logging: &CompilationLogging, name: &str) -> usize {
    logging
      .get(name)
      .map(|entries| {
        entries
          .iter()
          .filter(|entry| matches!(entry, LogType::Warn { .. }))
          .count()
      })
      .unwrap_or_default()
  }

  fn apply_scope_updates(
    storage: &mut MemoryStorage,
    scope: &'static str,
    mut updates: StorageUpdates,
  ) {
    for (key, value) in updates.remove(scope).unwrap_or_default() {
      if let Some(value) = value {
        storage.set(scope, key, value);
      } else {
        storage.remove(scope, &key);
      }
    }
  }

  #[tokio::test]
  async fn build_dependencies_test() {
    let scope = SnapshotScope::BUILD.name();
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
    let mut storage = MemoryStorage::default();
    let codec = Arc::new(CacheCodec::new(None));
    let snapshot = Arc::new(Snapshot::new(SnapshotOptions::default(), fs.clone(), codec));

    let mut build_deps = BuildDeps::new(&options, fs.clone(), snapshot.clone());

    let (logger, logging) = test_logger("test");
    let update_task = build_deps.save_updates_task(Vec::new(), logger);
    assert_eq!(warn_count(&logging, "test"), 0);
    apply_scope_updates(&mut storage, scope, update_task.await);
    assert_eq!(warn_count(&logging, "test"), 1);
    let data = storage.load(scope).await.expect("should load success");
    assert_eq!(data.len(), 9);

    let mut build_deps = BuildDeps::new(&options, fs.clone(), snapshot.clone());

    fs.write("/b.js".into(), r#"require("./c")"#.as_bytes())
      .await
      .unwrap();
    let validate_result = build_deps
      .validate(&storage)
      .await
      .expect("should validate success");
    assert!(matches!(
      validate_result,
      BuildDepsValidationResult::Invalid { .. }
    ));
    storage.reset(scope);

    let data = storage.load(scope).await.expect("should load success");
    assert_eq!(data.len(), 0);
    let (logger, logging) = test_logger("test");
    let update_task = build_deps.save_updates_task(Vec::new(), logger);
    apply_scope_updates(&mut storage, scope, update_task.await);
    assert_eq!(warn_count(&logging, "test"), 0);
    let data = storage.load(scope).await.expect("should load success");
    assert_eq!(data.len(), 10);
  }
}

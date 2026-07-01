mod option;
mod scope;
mod strategy;

use std::sync::Arc;

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_parallel::TryFutureConsumer;
use rspack_paths::{ArcPath, ArcPathSet};

use self::strategy::{StrategyHelper, ValidateResult};
pub use self::{
  option::{PathMatcher, SnapshotOptions, SnapshotStrategyOptions},
  scope::SnapshotScope,
  strategy::Strategy,
};
use super::{
  codec::CacheCodec,
  storage::{Storage, StorageUpdates},
};
use crate::FutureConsumer;

/// Snapshot is used to check if files have been modified or deleted.
///
/// Snapshot will generate `Strategy` for target file, and check the modification
/// through the generated `Strategy`
#[derive(Debug)]
pub struct Snapshot {
  options: Arc<SnapshotOptions>,
  fs: Arc<dyn ReadableFileSystem>,
  codec: Arc<CacheCodec>,
}

impl Snapshot {
  pub fn new(
    options: SnapshotOptions,
    fs: Arc<dyn ReadableFileSystem>,
    codec: Arc<CacheCodec>,
  ) -> Self {
    Self {
      options: Arc::new(options),
      fs,
      codec,
    }
  }

  async fn calc_strategy(
    options: &Arc<SnapshotOptions>,
    helper: &Arc<StrategyHelper>,
    path: &ArcPath,
    scope: SnapshotScope,
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
    Some(match scope {
      SnapshotScope::FILE => {
        helper
          .file_strategy(path, options.dependencies_strategy())
          .await
      }
      SnapshotScope::MISSING => Strategy::Missing,
      SnapshotScope::CONTEXT => {
        helper
          .dir_strategy(path, options.context_dependencies_strategy())
          .await
      }
      SnapshotScope::BUILD => {
        helper
          .dir_strategy(path, SnapshotStrategyOptions::hash())
          .await
      }
    })
  }

  #[tracing::instrument("Cache::Snapshot::reset", skip_all)]
  pub fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SnapshotScope::FILE.name());
    storage.reset(SnapshotScope::CONTEXT.name());
    storage.reset(SnapshotScope::MISSING.name());
  }

  async fn add_changes(
    &self,
    scope: SnapshotScope,
    paths: Vec<ArcPath>,
  ) -> Vec<(Vec<u8>, Vec<u8>)> {
    let helper = Arc::new(StrategyHelper::new(self.fs.clone(), self.options.clone()));
    let codec = self.codec.clone();
    let mut changes = Vec::new();
    // TODO merge package version file
    paths
      .into_iter()
      .map(|path| {
        let helper = helper.clone();
        let options = self.options.clone();
        let codec = codec.clone();
        async move {
          let strategy = Self::calc_strategy(&options, &helper, &path, scope).await?;
          Some((
            codec.encode(&path).expect("should encode success"),
            codec.encode(&strategy).expect("should encode success"),
          ))
        }
      })
      .fut_consume(|data| {
        if let Some((key, value)) = data {
          changes.push((key, value));
        }
      })
      .await;
    changes
  }

  pub async fn save_scope_updates(
    &self,
    scope: SnapshotScope,
    added: Vec<ArcPath>,
    removed: Vec<ArcPath>,
  ) -> StorageUpdates {
    let mut scope_updates: Vec<_> = removed
      .into_iter()
      .map(|item| (item.as_os_str().as_encoded_bytes().to_vec(), None))
      .collect();
    scope_updates.extend(
      self
        .add_changes(scope, added)
        .await
        .into_iter()
        .map(|(key, value)| (key, Some(value))),
    );

    let mut updates = StorageUpdates::default();
    if !scope_updates.is_empty() {
      updates.insert(
        scope.name().to_string(),
        scope_updates.into_iter().collect(),
      );
    }
    updates
  }

  pub async fn save_updates_task(
    self: Arc<Self>,
    file_deps: (Vec<ArcPath>, Vec<ArcPath>),
    context_deps: (Vec<ArcPath>, Vec<ArcPath>),
    missing_deps: (Vec<ArcPath>, Vec<ArcPath>),
  ) -> StorageUpdates {
    let mut updates = StorageUpdates::default();
    let (file_added, file_removed) = file_deps;
    let (context_added, context_removed) = context_deps;
    let (missing_added, missing_removed) = missing_deps;

    updates.extend(
      self
        .save_scope_updates(SnapshotScope::FILE, file_added, file_removed)
        .await,
    );
    updates.extend(
      self
        .save_scope_updates(SnapshotScope::CONTEXT, context_added, context_removed)
        .await,
    );
    updates.extend(
      self
        .save_scope_updates(SnapshotScope::MISSING, missing_added, missing_removed)
        .await,
    );

    updates
  }

  #[allow(clippy::type_complexity)]
  #[tracing::instrument("Cache::Snapshot::calc_modified_path", skip_all)]
  pub async fn calc_modified_paths(
    &self,
    storage: &dyn Storage,
    scope: SnapshotScope,
  ) -> Result<(bool, ArcPathSet, ArcPathSet, ArcPathSet)> {
    let mut modified_path = ArcPathSet::default();
    let mut deleted_path = ArcPathSet::default();
    let mut no_change_path = ArcPathSet::default();
    let helper = Arc::new(StrategyHelper::new(self.fs.clone(), self.options.clone()));
    let codec = self.codec.clone();

    let data = storage.load(scope.name()).await?;
    let is_hot_start = !data.is_empty();
    data
      .into_iter()
      .map(|(key, value)| {
        let helper = helper.clone();
        let codec = codec.clone();
        async move {
          let path = codec.decode::<ArcPath>(&key)?;
          let validate = match codec.decode::<Strategy>(&value) {
            Ok(strategy) => helper.validate(&path, &strategy).await,
            Err(_) => ValidateResult::Modified,
          };
          Ok::<_, rspack_error::Error>((path, validate))
        }
      })
      .try_fut_consume(|(path, validate)| match validate {
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
      .await?;

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
    Snapshot, SnapshotOptions, SnapshotScope,
  };
  use crate::cache::persistent::storage::Storage;

  macro_rules! p {
    ($tt:tt) => {
      ArcPath::from(std::path::Path::new($tt))
    };
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn should_generate_snapshot_save_updates() {
    let fs = Arc::new(MemoryFileSystem::default());
    let mut storage = MemoryStorage::default();
    let codec = Arc::new(CacheCodec::new(None));
    let snapshot = Arc::new(Snapshot::new(SnapshotOptions::default(), fs.clone(), codec));

    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/file1".into(), "abc".as_bytes()).await.unwrap();

    let mut updates = snapshot
      .clone()
      .save_updates_task(
        (vec![p!("/file1")], vec![]),
        (vec![], vec![]),
        (vec![], vec![]),
      )
      .await;
    for (key, value) in updates.remove(SnapshotScope::FILE.name()).unwrap() {
      if let Some(value) = value {
        storage.set(SnapshotScope::FILE.name(), key, value);
      } else {
        storage.remove(SnapshotScope::FILE.name(), &key);
      }
    }

    let (is_hot_start, modified_paths, deleted_paths, no_change_paths) = snapshot
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(is_hot_start);
    assert!(modified_paths.is_empty());
    assert!(deleted_paths.is_empty());
    assert!(no_change_paths.contains(&p!("/file1")));

    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/file1".into(), "abcd".as_bytes()).await.unwrap();
    let (_, modified_paths, deleted_paths, _) = snapshot
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(modified_paths.contains(&p!("/file1")));
    assert!(deleted_paths.is_empty());
  }
}

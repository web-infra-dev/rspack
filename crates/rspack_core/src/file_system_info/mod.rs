mod options;
mod snapshot;

use std::{
  hash::{Hash, Hasher},
  sync::Arc,
};

use futures::future::join3;
use rspack_error::Result;
use rspack_fs::{Error as FileSystemError, FileMetadata, ReadableFileSystem};
use rspack_parallel::TryFutureConsumer;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathSet};
use rustc_hash::FxHasher;
use simd_json::prelude::{ValueAsScalar, ValueObjectAccess};

use self::snapshot::{ManagedItemInfo, SnapshotEntry, TimestampAndHash};
pub use self::{
  options::{PathMatcher, SnapshotOptions, SnapshotStrategyOptions},
  snapshot::Snapshot,
};

#[derive(Debug, Default)]
pub(crate) struct SnapshotChanges {
  pub modified_files: InternedPathSet,
  pub removed_files: InternedPathSet,
}

#[derive(Debug)]
enum CapturedFile {
  Ignored,
  Managed {
    path: InternedPath,
    item: ManagedItemInfo,
  },
  Timestamp(SnapshotEntry<Option<u64>>),
  Hash(SnapshotEntry<Option<u64>>),
  TimestampAndHash(SnapshotEntry<Option<TimestampAndHash>>),
}

#[derive(Debug)]
enum CapturedContext {
  Ignored,
  Managed {
    path: InternedPath,
    item: ManagedItemInfo,
  },
  Timestamp(SnapshotEntry<Option<u64>>),
  Hash(SnapshotEntry<Option<u64>>),
  TimestampAndHash(SnapshotEntry<Option<TimestampAndHash>>),
}

#[derive(Debug)]
enum CapturedMissing {
  Ignored,
  Managed {
    path: InternedPath,
    item: ManagedItemInfo,
  },
  Existence(SnapshotEntry<bool>),
}

/// Creates and validates filesystem snapshots.
///
/// This module follows webpack's `FileSystemInfo` seam: callers provide paths
/// and a snapshot mode, while filesystem classification, capture, merge, and
/// validation stay behind this interface.
#[derive(Debug, Clone)]
pub struct FileSystemInfo {
  fs: Arc<dyn ReadableFileSystem>,
  options: Arc<SnapshotOptions>,
}

impl FileSystemInfo {
  pub fn new(fs: Arc<dyn ReadableFileSystem>, options: SnapshotOptions) -> Self {
    Self {
      fs,
      options: Arc::new(options),
    }
  }

  /// Mirrors webpack's `FileSystemInfo.createSnapshot` data flow.
  ///
  /// <https://github.com/webpack/webpack/blob/main/lib/FileSystemInfo.js#L2431-L2986>
  #[tracing::instrument("Cache::FileSystemInfo::create_snapshot", skip_all)]
  pub async fn create_snapshot(
    &self,
    start_time: Option<u64>,
    files: impl Iterator<Item = InternedPath>,
    contexts: impl Iterator<Item = InternedPath>,
    missing: impl Iterator<Item = InternedPath>,
    options: SnapshotStrategyOptions,
  ) -> Result<Snapshot> {
    let (file_snapshot, context_snapshot, missing_snapshot) = join3(
      self.capture_files(files, options),
      self.capture_contexts(contexts, options),
      self.capture_missing(missing),
    )
    .await;
    let mut file_snapshot = file_snapshot?;
    file_snapshot.start_time = start_time;
    file_snapshot.merge(context_snapshot?);
    file_snapshot.merge(missing_snapshot?);
    Ok(file_snapshot)
  }

  pub(crate) async fn create_module_snapshot(
    &self,
    start_time: Option<u64>,
    files: impl Iterator<Item = InternedPath>,
    contexts: impl Iterator<Item = InternedPath>,
    missing: impl Iterator<Item = InternedPath>,
  ) -> Result<Snapshot> {
    self
      .create_snapshot(
        start_time,
        files,
        contexts,
        missing,
        self.options.dependencies_strategy(),
      )
      .await
  }

  /// Mirrors webpack's `FileSystemInfo.mergeSnapshots`.
  pub fn merge_snapshots(&self, mut snapshot: Snapshot, added: Snapshot) -> Snapshot {
    snapshot.merge(added);
    snapshot
  }

  pub(crate) async fn collect_snapshot_changes(
    &self,
    snapshot: &Snapshot,
  ) -> Result<SnapshotChanges> {
    let mut changes = SnapshotChanges::default();

    for entry in &snapshot.file_timestamps {
      record_change(
        &mut changes,
        &entry.path,
        entry.value,
        self.modified_time(&entry.path).await?,
      );
    }
    for entry in &snapshot.file_hashes {
      record_change(
        &mut changes,
        &entry.path,
        entry.value,
        self.file_hash(&entry.path).await?.map(|value| value.hash),
      );
    }
    for entry in &snapshot.file_tshs {
      record_timestamp_and_hash_change(
        &mut changes,
        &entry.path,
        entry.value.clone(),
        self.file_hash(&entry.path).await?,
      );
    }
    for entry in &snapshot.context_timestamps {
      record_change(
        &mut changes,
        &entry.path,
        entry.value,
        self.context_timestamp_hash(&entry.path).await?,
      );
    }
    for entry in &snapshot.context_hashes {
      record_change(
        &mut changes,
        &entry.path,
        entry.value,
        self.context_hash(&entry.path).await?,
      );
    }
    for entry in &snapshot.context_tshs {
      let current = self.context_timestamp_and_hash(&entry.path).await?;
      record_timestamp_and_hash_change(&mut changes, &entry.path, entry.value.clone(), current);
    }
    for entry in &snapshot.missing_existence {
      let current = self.exists(&entry.path).await?;
      if current != entry.value {
        if entry.value && !current {
          changes.removed_files.insert(entry.path.clone());
        } else {
          changes.modified_files.insert(entry.path.clone());
        }
      }
    }

    self
      .check_managed_paths(snapshot, &snapshot.managed_files, false, &mut changes)
      .await?;
    self
      .check_managed_paths(snapshot, &snapshot.managed_contexts, false, &mut changes)
      .await?;
    self
      .check_managed_paths(snapshot, &snapshot.managed_missing, true, &mut changes)
      .await?;

    Ok(changes)
  }

  async fn capture_files(
    &self,
    files: impl Iterator<Item = InternedPath>,
    options: SnapshotStrategyOptions,
  ) -> Result<Snapshot> {
    let mut snapshot = Snapshot::default();
    let file_system_info = self.clone();
    files
      .map(move |path| {
        let file_system_info = file_system_info.clone();
        async move { file_system_info.capture_file(path, options).await }
      })
      .try_fut_consume(|entry| match entry {
        CapturedFile::Ignored => {}
        CapturedFile::Managed { path, item } => {
          snapshot.managed_files.insert(path);
          snapshot.insert_managed_item(item);
        }
        CapturedFile::Timestamp(entry) => snapshot.file_timestamps.push(entry),
        CapturedFile::Hash(entry) => snapshot.file_hashes.push(entry),
        CapturedFile::TimestampAndHash(entry) => snapshot.file_tshs.push(entry),
      })
      .await?;
    Ok(snapshot)
  }

  async fn capture_contexts(
    &self,
    contexts: impl Iterator<Item = InternedPath>,
    options: SnapshotStrategyOptions,
  ) -> Result<Snapshot> {
    let mut snapshot = Snapshot::default();
    let file_system_info = self.clone();
    contexts
      .map(move |path| {
        let file_system_info = file_system_info.clone();
        async move { file_system_info.capture_context(path, options).await }
      })
      .try_fut_consume(|entry| match entry {
        CapturedContext::Ignored => {}
        CapturedContext::Managed { path, item } => {
          snapshot.managed_contexts.insert(path);
          snapshot.insert_managed_item(item);
        }
        CapturedContext::Timestamp(entry) => snapshot.context_timestamps.push(entry),
        CapturedContext::Hash(entry) => snapshot.context_hashes.push(entry),
        CapturedContext::TimestampAndHash(entry) => snapshot.context_tshs.push(entry),
      })
      .await?;
    Ok(snapshot)
  }

  async fn capture_missing(&self, missing: impl Iterator<Item = InternedPath>) -> Result<Snapshot> {
    let mut snapshot = Snapshot::default();
    let file_system_info = self.clone();
    missing
      .map(move |path| {
        let file_system_info = file_system_info.clone();
        async move { file_system_info.capture_missing_path(path).await }
      })
      .try_fut_consume(|entry| match entry {
        CapturedMissing::Ignored => {}
        CapturedMissing::Managed { path, item } => {
          snapshot.managed_missing.insert(path);
          snapshot.insert_managed_item(item);
        }
        CapturedMissing::Existence(entry) => snapshot.missing_existence.push(entry),
      })
      .await?;
    Ok(snapshot)
  }

  async fn capture_file(
    &self,
    path: InternedPath,
    options: SnapshotStrategyOptions,
  ) -> Result<CapturedFile> {
    if self.is_immutable(&path) {
      return Ok(CapturedFile::Ignored);
    }
    if self.is_managed(&path)
      && let Some(item) = self.managed_item_info(&path).await?
    {
      return Ok(CapturedFile::Managed { path, item });
    }
    Ok(if options.hash && options.timestamp {
      let value = self.file_hash(&path).await?;
      CapturedFile::TimestampAndHash(SnapshotEntry { path, value })
    } else if options.hash {
      let value = self.file_hash(&path).await?.map(|value| value.hash);
      CapturedFile::Hash(SnapshotEntry { path, value })
    } else {
      let value = self.modified_time(&path).await?;
      CapturedFile::Timestamp(SnapshotEntry { path, value })
    })
  }

  async fn capture_context(
    &self,
    path: InternedPath,
    options: SnapshotStrategyOptions,
  ) -> Result<CapturedContext> {
    if self.is_immutable(&path) {
      return Ok(CapturedContext::Ignored);
    }
    if self.is_managed(&path)
      && let Some(item) = self.managed_item_info(&path).await?
    {
      return Ok(CapturedContext::Managed { path, item });
    }
    Ok(if options.hash && options.timestamp {
      let value = self.context_timestamp_and_hash(&path).await?;
      CapturedContext::TimestampAndHash(SnapshotEntry { path, value })
    } else if options.hash {
      let value = self.context_hash(&path).await?;
      CapturedContext::Hash(SnapshotEntry { path, value })
    } else {
      let value = self.context_timestamp_hash(&path).await?;
      CapturedContext::Timestamp(SnapshotEntry { path, value })
    })
  }

  async fn capture_missing_path(&self, path: InternedPath) -> Result<CapturedMissing> {
    if self.is_immutable(&path) {
      return Ok(CapturedMissing::Ignored);
    }
    if self.is_managed(&path)
      && let Some(item) = self.managed_item_info(&path).await?
    {
      return Ok(CapturedMissing::Managed { path, item });
    }
    let value = self.exists(&path).await?;
    Ok(CapturedMissing::Existence(SnapshotEntry { path, value }))
  }

  async fn check_managed_paths(
    &self,
    snapshot: &Snapshot,
    paths: &InternedPathSet,
    expect_missing: bool,
    changes: &mut SnapshotChanges,
  ) -> Result<()> {
    for path in paths {
      if expect_missing && self.exists(path).await? {
        changes.modified_files.insert(path.clone());
        continue;
      }
      let Some(current) = self.managed_item_info(path).await? else {
        changes.removed_files.insert(path.clone());
        continue;
      };
      let valid = snapshot
        .managed_item_info
        .iter()
        .any(|item| item.path == current.path && item.version == current.version);
      if !valid {
        changes.modified_files.insert(path.clone());
      }
    }
    Ok(())
  }

  fn is_immutable(&self, path: &InternedPath) -> bool {
    self.options.is_immutable_path(&path.to_string_lossy())
  }

  fn is_managed(&self, path: &InternedPath) -> bool {
    self.options.is_managed_path(&path.to_string_lossy())
  }

  async fn exists(&self, path: &InternedPath) -> Result<bool> {
    Ok(self.metadata(path).await?.is_some())
  }

  async fn metadata(&self, path: &InternedPath) -> Result<Option<FileMetadata>> {
    missing_as_none(self.fs.metadata(path.assert_utf8()).await)
  }

  fn modified_time_from_metadata(metadata: &FileMetadata) -> u64 {
    metadata.ctime_ms.max(metadata.mtime_ms)
  }

  async fn modified_time(&self, path: &InternedPath) -> Result<Option<u64>> {
    Ok(
      self
        .metadata(path)
        .await?
        .map(|metadata| Self::modified_time_from_metadata(&metadata)),
    )
  }

  async fn file_hash(&self, path: &InternedPath) -> Result<Option<TimestampAndHash>> {
    let Some(metadata) = self.metadata(path).await? else {
      return Ok(None);
    };
    self.file_hash_from_metadata(path, &metadata).await
  }

  async fn file_hash_from_metadata(
    &self,
    path: &InternedPath,
    metadata: &FileMetadata,
  ) -> Result<Option<TimestampAndHash>> {
    let mut hasher = FxHasher::default();
    if metadata.is_symlink {
      let target = self.fs.canonicalize(path.assert_utf8()).await?;
      target.hash(&mut hasher);
    } else if metadata.is_file {
      let Some(content) = missing_as_none(self.fs.read(path.assert_utf8()).await)? else {
        return Ok(None);
      };
      content.hash(&mut hasher);
    }
    Ok(Some(TimestampAndHash {
      timestamp: Self::modified_time_from_metadata(metadata),
      hash: hasher.finish(),
    }))
  }

  #[async_recursion::async_recursion]
  async fn context_hash(&self, path: &InternedPath) -> Result<Option<u64>> {
    let Some(metadata) = self.metadata(path).await? else {
      return Ok(None);
    };
    if !metadata.is_directory || metadata.is_symlink {
      return Ok(
        self
          .file_hash_from_metadata(path, &metadata)
          .await?
          .map(|value| value.hash),
      );
    }
    let Some(mut children) = missing_as_none(self.fs.read_dir(path.assert_utf8()).await)? else {
      return Ok(None);
    };
    children.sort();
    let mut hasher = FxHasher::default();
    for child in children {
      child.hash(&mut hasher);
      let child_path = InternedPath::from(path.join(child));
      if self.is_immutable(&child_path) {
        continue;
      }
      if self.is_managed(&child_path)
        && let Some(item) = self.managed_item_info(&child_path).await?
      {
        item.path.hash(&mut hasher);
        item.version.hash(&mut hasher);
        continue;
      }
      let Some(hash) = self.context_hash(&child_path).await? else {
        return Ok(None);
      };
      hash.hash(&mut hasher);
    }
    Ok(Some(hasher.finish()))
  }

  #[async_recursion::async_recursion]
  async fn context_timestamp_hash(&self, path: &InternedPath) -> Result<Option<u64>> {
    let Some(metadata) = self.metadata(path).await? else {
      return Ok(None);
    };
    if !metadata.is_directory || metadata.is_symlink {
      return Ok(Some(Self::modified_time_from_metadata(&metadata)));
    }
    let Some(mut children) = missing_as_none(self.fs.read_dir(path.assert_utf8()).await)? else {
      return Ok(None);
    };
    children.sort();
    let mut hasher = FxHasher::default();
    for child in children {
      child.hash(&mut hasher);
      let child_path = InternedPath::from(path.join(child));
      if self.is_immutable(&child_path) {
        continue;
      }
      if self.is_managed(&child_path)
        && let Some(item) = self.managed_item_info(&child_path).await?
      {
        item.path.hash(&mut hasher);
        item.version.hash(&mut hasher);
        continue;
      }
      let Some(timestamp) = self.context_timestamp_hash(&child_path).await? else {
        return Ok(None);
      };
      timestamp.hash(&mut hasher);
    }
    Ok(Some(hasher.finish()))
  }

  async fn context_timestamp_and_hash(
    &self,
    path: &InternedPath,
  ) -> Result<Option<TimestampAndHash>> {
    let (timestamp, hash) =
      futures::try_join!(self.context_timestamp_hash(path), self.context_hash(path))?;
    Ok(match (timestamp, hash) {
      (Some(timestamp), Some(hash)) => Some(TimestampAndHash { timestamp, hash }),
      _ => None,
    })
  }

  async fn managed_item_info(&self, path: &InternedPath) -> Result<Option<ManagedItemInfo>> {
    let mut current = match self.metadata(path).await? {
      Some(metadata) if !metadata.is_directory => match path.parent() {
        Some(parent) => InternedPath::from(parent),
        None => return Ok(None),
      },
      _ => path.clone(),
    };
    loop {
      let package_json = InternedPath::from(current.join("package.json"));
      if let Some(mut content) =
        missing_package_json_as_none(self.fs.read(package_json.assert_utf8()).await)?
        && let Ok(value) = simd_json::to_borrowed_value(&mut content)
        && let Some(version) = value.get("version").and_then(|value| value.as_str())
      {
        return Ok(Some(ManagedItemInfo {
          path: current,
          version: version.to_string(),
        }));
      }
      let Some(parent) = current.parent() else {
        return Ok(None);
      };
      current = InternedPath::from(parent);
    }
  }
}

fn missing_package_json_as_none<T>(result: rspack_fs::Result<T>) -> Result<Option<T>> {
  match result {
    Ok(value) => Ok(Some(value)),
    Err(FileSystemError::Io(error))
      if matches!(
        error.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
      ) =>
    {
      Ok(None)
    }
    Err(error) => Err(error.into()),
  }
}

fn missing_as_none<T>(result: rspack_fs::Result<T>) -> Result<Option<T>> {
  match result {
    Ok(value) => Ok(Some(value)),
    Err(FileSystemError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(error) => Err(error.into()),
  }
}

fn record_change<T: PartialEq>(
  changes: &mut SnapshotChanges,
  path: &InternedPath,
  previous: Option<T>,
  current: Option<T>,
) {
  if previous == current {
    return;
  }
  if previous.is_some() && current.is_none() {
    changes.removed_files.insert(path.clone());
  } else {
    changes.modified_files.insert(path.clone());
  }
}

fn record_timestamp_and_hash_change(
  changes: &mut SnapshotChanges,
  path: &InternedPath,
  previous: Option<TimestampAndHash>,
  current: Option<TimestampAndHash>,
) {
  match (previous, current) {
    (None, None) => {}
    (Some(_), None) => {
      changes.removed_files.insert(path.clone());
    }
    (Some(previous), Some(current))
      if previous.timestamp == current.timestamp || previous.hash == current.hash => {}
    _ => {
      changes.modified_files.insert(path.clone());
    }
  }
}

#[cfg(test)]
mod tests;

use std::{collections::VecDeque, fmt, sync::Arc};

use rspack_error::{Result, error};
use rspack_fs::{Error as FsError, FileMetadata, ReadableFileSystem};
use rspack_hash::{HashDigest, HashFunction, RspackHashDigest, RspackHasher};
use rspack_parallel::TryFutureConsumer;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathDashMap, InternedPathSet, Utf8Path};
use rspack_regex::RspackRegex;
use rspack_util::{node_path::NodePath, time::mtime_accuracy};
use simd_json::prelude::{ValueAsScalar, ValueObjectAccess};

use super::{
  ContextFileSystemInfoEntry, ContextTimestampAndHash, FileHash, FileSystemInfoEntry, Snapshot,
  TimestampAndHash,
};
use crate::{
  CompilationLogger, InfrastructureLogger, LogType, Logger, PathMatcher, SnapshotOptions,
  SnapshotStrategyOptions,
  cache::{BuildDependencyHelper, is_node_package_path},
};

#[derive(Debug, Clone)]
pub(crate) enum FileSystemInfoLogger {
  Compilation(CompilationLogger),
  Infrastructure(InfrastructureLogger),
}

impl From<CompilationLogger> for FileSystemInfoLogger {
  fn from(logger: CompilationLogger) -> Self {
    Self::Compilation(logger)
  }
}

impl From<InfrastructureLogger> for FileSystemInfoLogger {
  fn from(logger: InfrastructureLogger) -> Self {
    Self::Infrastructure(logger)
  }
}

impl Logger for FileSystemInfoLogger {
  fn raw(&self, log_type: LogType) {
    match self {
      Self::Compilation(logger) => logger.raw(log_type),
      Self::Infrastructure(logger) => logger.raw(log_type),
    }
  }
}

#[derive(Debug, Default)]
pub struct ResolvedBuildDependencies {
  pub(crate) files: InternedPathSet,
  pub(crate) contexts: InternedPathSet,
  pub(crate) missing: InternedPathSet,
}

#[derive(Debug)]
pub enum SnapshotValidationResult {
  Valid,
  Invalid {
    modified_files: InternedPathSet,
    removed_files: InternedPathSet,
  },
}

#[derive(Debug, Clone, Copy)]
enum SnapshotMode {
  Timestamp,
  Hash,
  TimestampAndHash,
}

impl From<SnapshotStrategyOptions> for SnapshotMode {
  fn from(value: SnapshotStrategyOptions) -> Self {
    if value.hash {
      if value.timestamp {
        Self::TimestampAndHash
      } else {
        Self::Hash
      }
    } else {
      // This matches webpack: timestamp is the fallback mode even when both
      // option bits are false.
      Self::Timestamp
    }
  }
}

/// The path categories checked by webpack's `createSnapshot` -> `checkManaged`.
/// Cloning a cached classification only copies the interned managed-path handle.
#[derive(Clone)]
enum PathClassification {
  Unmanaged,
  Immutable,
  Managed(InternedPath),
}

#[derive(Debug)]
struct ContextValue {
  safe_time: u64,
  timestamp_hash: Option<RspackHashDigest>,
  hash: Option<RspackHashDigest>,
}

/// Cached access to filesystem state and snapshot algorithms.
///
/// The module follows webpack's `FileSystemInfo` seam: callers provide path
/// sets and a strategy, while path classification, managed package handling,
/// timestamp accuracy, hashing, merging and validation stay behind this
/// interface.
///
/// See webpack's `FileSystemInfo` implementation:
/// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L1282-L1450
#[derive(Clone)]
pub struct FileSystemInfo {
  inner: Arc<FileSystemInfoInner>,
}

struct FileSystemInfoInner {
  fs: Arc<dyn ReadableFileSystem>,
  logger: FileSystemInfoLogger,
  module: SnapshotStrategyOptions,
  build_dependencies: SnapshotStrategyOptions,
  unmanaged_paths_with_slash: Vec<String>,
  unmanaged_paths_reg_exps: Vec<RspackRegex>,
  managed_paths_with_slash: Vec<String>,
  managed_paths_reg_exps: Vec<RspackRegex>,
  immutable_paths_with_slash: Vec<String>,
  immutable_paths_reg_exps: Vec<RspackRegex>,
  path_classification_cache: InternedPathDashMap<PathClassification>,
  hash_function: HashFunction,
  file_timestamps: InternedPathDashMap<Option<FileSystemInfoEntry>>,
  file_hashes: InternedPathDashMap<Option<FileHash>>,
  file_timestamp_hashes: InternedPathDashMap<Option<TimestampAndHash>>,
  context_timestamps: InternedPathDashMap<Option<ContextFileSystemInfoEntry>>,
  context_hashes: InternedPathDashMap<Option<RspackHashDigest>>,
  context_timestamp_hashes: InternedPathDashMap<Option<ContextTimestampAndHash>>,
  managed_items: InternedPathDashMap<Option<String>>,
  managed_item_directory_info: InternedPathDashMap<Arc<InternedPathSet>>,
}

impl fmt::Debug for FileSystemInfo {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FileSystemInfo")
      .finish_non_exhaustive()
  }
}

impl FileSystemInfo {
  pub(crate) fn new(
    fs: Arc<dyn ReadableFileSystem>,
    logger: impl Into<FileSystemInfoLogger>,
    options: SnapshotOptions,
    hash_function: HashFunction,
  ) -> Self {
    let (unmanaged_paths_with_slash, unmanaged_paths_reg_exps) =
      split_snapshot_paths(options.unmanaged_paths);
    let (managed_paths_with_slash, managed_paths_reg_exps) =
      split_snapshot_paths(options.managed_paths);
    let (immutable_paths_with_slash, immutable_paths_reg_exps) =
      split_snapshot_paths(options.immutable_paths);
    Self {
      inner: Arc::new(FileSystemInfoInner {
        fs,
        logger: logger.into(),
        module: options.module,
        build_dependencies: options.build_dependencies,
        unmanaged_paths_with_slash,
        unmanaged_paths_reg_exps,
        managed_paths_with_slash,
        managed_paths_reg_exps,
        immutable_paths_with_slash,
        immutable_paths_reg_exps,
        path_classification_cache: Default::default(),
        hash_function,
        file_timestamps: Default::default(),
        file_hashes: Default::default(),
        file_timestamp_hashes: Default::default(),
        context_timestamps: Default::default(),
        context_hashes: Default::default(),
        context_timestamp_hashes: Default::default(),
        managed_items: Default::default(),
        managed_item_directory_info: Default::default(),
      }),
    }
  }

  /// Corresponds to webpack's `FileSystemInfo.createSnapshot`: classify paths,
  /// snapshot unmanaged paths, then read each managed item's metadata once.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L2534-L3079
  pub async fn create_snapshot(
    &self,
    start_time: Option<u64>,
    files: &InternedPathSet,
    contexts: &InternedPathSet,
    missing: &InternedPathSet,
    strategy: SnapshotStrategyOptions,
  ) -> Result<Snapshot> {
    let mut snapshot = Snapshot {
      start_time,
      ..Default::default()
    };
    let mode = strategy.into();

    let mut managed_files = InternedPathSet::default();
    let mut managed_contexts = InternedPathSet::default();
    let mut managed_missing = InternedPathSet::default();
    let mut managed_items = InternedPathSet::default();
    let files = self.capture_non_managed(files, &mut managed_files, &mut managed_items);
    let contexts = self.capture_non_managed(contexts, &mut managed_contexts, &mut managed_items);
    let missing = self.capture_non_managed(missing, &mut managed_missing, &mut managed_items);

    self
      .process_captured_files(&mut snapshot, files, mode)
      .await?;
    self
      .process_captured_directories(&mut snapshot, contexts, mode)
      .await?;
    self
      .process_captured_missing(&mut snapshot, missing)
      .await?;

    // The managedItems loop at the end of webpack's createSnapshot.
    // https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L3018-L3075
    for path in managed_items {
      if let Some(info) = self.get_managed_item_info(&path).await? {
        if !info.starts_with('*') {
          managed_files.insert(InternedPath::from(path.join("package.json")));
        } else if info == "*nested" {
          managed_missing.insert(InternedPath::from(path.join("package.json")));
        }
        snapshot
          .managed_item_info
          .get_or_insert_default()
          .insert(path, info);
      } else {
        // Fall back to normal snapshotting for paths under this managed item.
        let path_str = path.to_string_lossy();
        let capture = |paths: &InternedPathSet| {
          paths
            .iter()
            .filter(|file| file.to_string_lossy().starts_with(path_str.as_ref()))
            .cloned()
            .collect()
        };
        self
          .process_captured_files(&mut snapshot, capture(&managed_files), mode)
          .await?;
        self
          .process_captured_directories(&mut snapshot, capture(&managed_contexts), mode)
          .await?;
        self
          .process_captured_missing(&mut snapshot, capture(&managed_missing))
          .await?;
      }
    }
    if !managed_files.is_empty() {
      snapshot.managed_files = Some(managed_files);
    }
    if !managed_contexts.is_empty() {
      snapshot.managed_contexts = Some(managed_contexts);
    }
    if !managed_missing.is_empty() {
      snapshot.managed_missing = Some(managed_missing);
    }
    Ok(snapshot)
  }

  /// See webpack's snapshot merge implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L3081-L3166
  pub fn merge_snapshots(&self, mut first: Snapshot, second: Snapshot) -> Snapshot {
    first.merge(second);
    first
  }

  pub fn module_strategy(&self) -> SnapshotStrategyOptions {
    self.inner.module
  }

  pub fn build_dependencies_strategy(&self) -> SnapshotStrategyOptions {
    self.inner.build_dependencies
  }

  /// See webpack's snapshot validation implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L3168-L3735
  pub async fn check_snapshot_valid(
    &self,
    snapshot: &Snapshot,
  ) -> Result<SnapshotValidationResult> {
    let mut modified_files = InternedPathSet::default();
    let mut removed_files = InternedPathSet::default();
    self
      .validate_snapshot(snapshot, &mut modified_files, &mut removed_files)
      .await?;
    if modified_files.is_empty() && removed_files.is_empty() {
      Ok(SnapshotValidationResult::Valid)
    } else {
      Ok(SnapshotValidationResult::Invalid {
        modified_files,
        removed_files,
      })
    }
  }

  /// Resolve build dependencies that are not in the current snapshot.
  ///
  /// For performance reasons, recursive searches stop at dependencies in
  /// `node_modules`.
  ///
  /// See webpack's build dependency resolution implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L1873-L2523
  pub async fn resolve_build_dependencies(
    &self,
    paths: impl Iterator<Item = InternedPath>,
  ) -> ResolvedBuildDependencies {
    let mut helper = BuildDependencyHelper::new(self.inner.fs.clone(), self.inner.logger.clone());
    let mut resolved = ResolvedBuildDependencies::default();
    let mut visited = InternedPathSet::default();
    let mut queue = VecDeque::new();
    queue.extend(paths);

    while let Some(dependency) = queue.pop_front() {
      if !visited.insert(dependency.clone()) {
        continue;
      }
      match self.inner.fs.metadata(dependency.assert_utf8()).await {
        Ok(metadata) if metadata.is_directory => {
          resolved.contexts.insert(dependency.clone());
        }
        Ok(_) => {
          resolved.files.insert(dependency.clone());
        }
        Err(_) => {
          resolved.missing.insert(dependency.clone());
        }
      }
      if is_node_package_path(&dependency) {
        continue;
      }
      if let Some(children) = helper.resolve(dependency.assert_utf8()).await {
        queue.extend(
          children
            .into_iter()
            .map(|path| InternedPath::from(path.as_path())),
        );
      }
    }

    resolved
  }

  /// Corresponds to `checkManaged` inside webpack's `FileSystemInfo.createSnapshot`.
  /// The classification is returned here so snapshot creation and context reads
  /// can share the same path checks.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L2627-L2687
  fn check_managed(&self, path: &InternedPath) -> PathClassification {
    let classification_cache = &self.inner.path_classification_cache;
    if let Some(cached) = classification_cache.get(path) {
      return cached.clone();
    }
    let path_str = path.to_string_lossy();
    for unmanaged_path in &self.inner.unmanaged_paths_reg_exps {
      if unmanaged_path.test(&path_str) {
        classification_cache.insert(path.clone(), PathClassification::Unmanaged);
        return PathClassification::Unmanaged;
      }
    }
    for unmanaged_path in &self.inner.unmanaged_paths_with_slash {
      if path_str.starts_with(unmanaged_path) {
        classification_cache.insert(path.clone(), PathClassification::Unmanaged);
        return PathClassification::Unmanaged;
      }
    }
    for immutable_path in &self.inner.immutable_paths_reg_exps {
      if immutable_path.test(&path_str) {
        classification_cache.insert(path.clone(), PathClassification::Immutable);
        return PathClassification::Immutable;
      }
    }
    for immutable_path in &self.inner.immutable_paths_with_slash {
      if path_str.starts_with(immutable_path) {
        classification_cache.insert(path.clone(), PathClassification::Immutable);
        return PathClassification::Immutable;
      }
    }
    for managed_path in &self.inner.managed_paths_reg_exps {
      // webpack passes `managedPath.exec(path)[1]` to getManagedItem.
      if let Some(managed_path) = managed_path.capture(&path_str, 1)
        && let Some(managed_item) = get_managed_item(managed_path, &path_str)
      {
        let managed_item = InternedPath::from(managed_item);
        classification_cache.insert(
          path.clone(),
          PathClassification::Managed(managed_item.clone()),
        );
        return PathClassification::Managed(managed_item);
      }
    }
    for managed_path in &self.inner.managed_paths_with_slash {
      if path_str.starts_with(managed_path)
        && let Some(managed_item) = get_managed_item(managed_path, &path_str)
      {
        let managed_item = InternedPath::from(managed_item);
        classification_cache.insert(
          path.clone(),
          PathClassification::Managed(managed_item.clone()),
        );
        return PathClassification::Managed(managed_item);
      }
    }
    classification_cache.insert(path.clone(), PathClassification::Unmanaged);
    PathClassification::Unmanaged
  }

  /// Corresponds to `captureNonManaged` inside webpack's `FileSystemInfo.createSnapshot`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L2694-L2701
  fn capture_non_managed(
    &self,
    items: &InternedPathSet,
    managed_set: &mut InternedPathSet,
    managed_items: &mut InternedPathSet,
  ) -> Vec<InternedPath> {
    let mut captured_items = Vec::with_capacity(items.len());
    for path in items {
      match self.check_managed(path) {
        PathClassification::Immutable => {
          managed_set.insert(path.clone());
        }
        PathClassification::Managed(managed_item) => {
          managed_items.insert(managed_item);
          managed_set.insert(path.clone());
        }
        PathClassification::Unmanaged => {
          captured_items.push(path.clone());
        }
      }
    }
    captured_items
  }

  /// Corresponds to `processCapturedFiles` inside webpack's `FileSystemInfo.createSnapshot`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L2706
  async fn process_captured_files(
    &self,
    snapshot: &mut Snapshot,
    paths: Vec<InternedPath>,
    mode: SnapshotMode,
  ) -> Result<()> {
    if paths.is_empty() {
      return Ok(());
    }
    match mode {
      SnapshotMode::Timestamp => {
        let map = snapshot.file_timestamps.get_or_insert_default();
        paths
          .into_iter()
          .map(|path| {
            let this = self.clone();
            async move {
              let value = this.file_timestamp(&path).await?;
              Ok::<_, rspack_error::Error>((path, value))
            }
          })
          .try_fut_consume(|(path, value)| {
            map.insert(path, value);
          })
          .await?;
      }
      SnapshotMode::Hash => {
        let map = snapshot.file_hashes.get_or_insert_default();
        paths
          .into_iter()
          .map(|path| {
            let this = self.clone();
            async move {
              let value = this.file_hash(&path).await?;
              Ok::<_, rspack_error::Error>((path, value))
            }
          })
          .try_fut_consume(|(path, value)| {
            map.insert(path, value);
          })
          .await?;
      }
      SnapshotMode::TimestampAndHash => {
        let map = snapshot.file_timestamp_hashes.get_or_insert_default();
        paths
          .into_iter()
          .map(|path| {
            let this = self.clone();
            async move {
              let value = this.file_timestamp_and_hash(&path).await?;
              Ok::<_, rspack_error::Error>((path, value))
            }
          })
          .try_fut_consume(|(path, value)| {
            map.insert(path, value);
          })
          .await?;
      }
    }
    Ok(())
  }

  /// Corresponds to `processCapturedDirectories` inside webpack's `FileSystemInfo.createSnapshot`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L2813
  async fn process_captured_directories(
    &self,
    snapshot: &mut Snapshot,
    paths: Vec<InternedPath>,
    mode: SnapshotMode,
  ) -> Result<()> {
    if paths.is_empty() {
      return Ok(());
    }
    match mode {
      SnapshotMode::Timestamp => {
        let map = snapshot.context_timestamps.get_or_insert_default();
        paths
          .into_iter()
          .map(|path| {
            let this = self.clone();
            async move {
              let value = this.context_timestamp(&path).await?;
              Ok::<_, rspack_error::Error>((path, value))
            }
          })
          .try_fut_consume(|(path, value)| {
            map.insert(path, value);
          })
          .await?;
      }
      SnapshotMode::Hash => {
        let map = snapshot.context_hashes.get_or_insert_default();
        paths
          .into_iter()
          .map(|path| {
            let this = self.clone();
            async move {
              let value = this.context_hash(&path).await?;
              Ok::<_, rspack_error::Error>((path, value))
            }
          })
          .try_fut_consume(|(path, value)| {
            map.insert(path, value);
          })
          .await?;
      }
      SnapshotMode::TimestampAndHash => {
        let map = snapshot.context_timestamp_hashes.get_or_insert_default();
        paths
          .into_iter()
          .map(|path| {
            let this = self.clone();
            async move {
              let value = this.context_timestamp_and_hash(&path).await?;
              Ok::<_, rspack_error::Error>((path, value))
            }
          })
          .try_fut_consume(|(path, value)| {
            map.insert(path, value);
          })
          .await?;
      }
    }
    Ok(())
  }

  /// Corresponds to `processCapturedMissing` inside webpack's `FileSystemInfo.createSnapshot`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L2984
  async fn process_captured_missing(
    &self,
    snapshot: &mut Snapshot,
    paths: Vec<InternedPath>,
  ) -> Result<()> {
    if paths.is_empty() {
      return Ok(());
    }
    let map = snapshot.missing_existence.get_or_insert_default();
    paths
      .into_iter()
      .map(|path| {
        let this = self.clone();
        async move {
          let exists = this.metadata(&path).await?.is_some();
          Ok::<_, rspack_error::Error>((path, exists))
        }
      })
      .try_fut_consume(|(path, exists)| {
        map.insert(path, exists);
      })
      .await
  }

  async fn metadata(&self, path: &InternedPath) -> Result<Option<FileMetadata>> {
    match self.inner.fs.metadata(path.assert_utf8()).await {
      Ok(metadata) => Ok(Some(metadata)),
      Err(error) if is_not_found(&error) => Ok(None),
      Err(error) => Err(error.into()),
    }
  }

  /// See webpack's file timestamp and hash implementations:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L3737-L3865
  async fn file_timestamp(&self, path: &InternedPath) -> Result<Option<FileSystemInfoEntry>> {
    if let Some(entry) = self.inner.file_timestamps.get(path) {
      return Ok(entry.clone());
    }
    let entry = self.metadata(path).await?.map(|metadata| {
      if metadata.is_directory {
        FileSystemInfoEntry {
          safe_time: 0,
          timestamp: None,
        }
      } else {
        let timestamp = metadata.mtime_ms;
        let accuracy = mtime_accuracy(timestamp);
        FileSystemInfoEntry {
          safe_time: if timestamp == 0 {
            u64::MAX
          } else {
            timestamp.saturating_add(accuracy)
          },
          timestamp: Some(timestamp),
        }
      }
    });
    self
      .inner
      .file_timestamps
      .insert(path.clone(), entry.clone());
    Ok(entry)
  }

  async fn file_hash(&self, path: &InternedPath) -> Result<Option<FileHash>> {
    if let Some(entry) = self.inner.file_hashes.get(path) {
      return Ok(entry.clone());
    }
    let Some(metadata) = self.metadata(path).await? else {
      self.inner.file_hashes.insert(path.clone(), None);
      return Ok(None);
    };
    let hash = if metadata.is_directory {
      FileHash::Directory
    } else {
      let content = match self.inner.fs.read(path.assert_utf8()).await {
        Ok(content) => content,
        Err(error) if is_not_found(&error) => {
          self.inner.file_hashes.insert(path.clone(), None);
          return Ok(None);
        }
        Err(error) => return Err(error.into()),
      };
      FileHash::Digest(self.digest(&content))
    };
    self
      .inner
      .file_hashes
      .insert(path.clone(), Some(hash.clone()));
    Ok(Some(hash))
  }

  async fn file_timestamp_and_hash(&self, path: &InternedPath) -> Result<Option<TimestampAndHash>> {
    if let Some(entry) = self.inner.file_timestamp_hashes.get(path) {
      return Ok(entry.clone());
    }
    let (Some(timestamp), Some(hash)) = (
      self.file_timestamp(path).await?,
      self.file_hash(path).await?,
    ) else {
      self.inner.file_timestamp_hashes.insert(path.clone(), None);
      return Ok(None);
    };
    let value = TimestampAndHash {
      safe_time: timestamp.safe_time,
      timestamp: timestamp.timestamp,
      hash,
    };
    self
      .inner
      .file_timestamp_hashes
      .insert(path.clone(), Some(value.clone()));
    Ok(Some(value))
  }

  async fn context_timestamp(
    &self,
    path: &InternedPath,
  ) -> Result<Option<ContextFileSystemInfoEntry>> {
    if let Some(entry) = self.inner.context_timestamps.get(path) {
      return Ok(entry.clone());
    }
    let mut visiting = InternedPathSet::default();
    let value = self
      .context_value(path, SnapshotMode::Timestamp, &mut visiting)
      .await?
      .map(|value| ContextFileSystemInfoEntry {
        safe_time: value.safe_time,
        timestamp_hash: value
          .timestamp_hash
          .expect("timestamp mode should produce a timestamp hash"),
      });
    self
      .inner
      .context_timestamps
      .insert(path.clone(), value.clone());
    Ok(value)
  }

  async fn context_hash(&self, path: &InternedPath) -> Result<Option<RspackHashDigest>> {
    if let Some(entry) = self.inner.context_hashes.get(path) {
      return Ok(entry.clone());
    }
    let mut visiting = InternedPathSet::default();
    let value = self
      .context_value(path, SnapshotMode::Hash, &mut visiting)
      .await?
      .map(|value| value.hash.expect("hash mode should produce a hash"));
    self
      .inner
      .context_hashes
      .insert(path.clone(), value.clone());
    Ok(value)
  }

  async fn context_timestamp_and_hash(
    &self,
    path: &InternedPath,
  ) -> Result<Option<ContextTimestampAndHash>> {
    if let Some(entry) = self.inner.context_timestamp_hashes.get(path) {
      return Ok(entry.clone());
    }
    let mut visiting = InternedPathSet::default();
    let value = self
      .context_value(path, SnapshotMode::TimestampAndHash, &mut visiting)
      .await?
      .map(|value| ContextTimestampAndHash {
        safe_time: value.safe_time,
        timestamp_hash: value
          .timestamp_hash
          .expect("timestamp and hash mode should produce a timestamp hash"),
        hash: value
          .hash
          .expect("timestamp and hash mode should produce a hash"),
      });
    self
      .inner
      .context_timestamp_hashes
      .insert(path.clone(), value.clone());
    Ok(value)
  }

  /// See webpack's recursive context timestamp and hash implementations:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L3867-L4490
  #[async_recursion::async_recursion]
  async fn context_value(
    &self,
    path: &InternedPath,
    mode: SnapshotMode,
    visiting: &mut InternedPathSet,
  ) -> Result<Option<ContextValue>> {
    if !visiting.insert(path.clone()) {
      return Ok(Some(self.context_leaf(
        path.to_string_lossy().as_bytes(),
        mode,
        0,
      )));
    }

    let result = self.read_context(path, mode, visiting).await;
    visiting.remove(path);
    result
  }

  /// Corresponds to webpack's `FileSystemInfo._readContext`, combining its
  /// timestamp and hash callbacks according to `mode`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L3881-L3977
  async fn read_context(
    &self,
    path: &InternedPath,
    mode: SnapshotMode,
    visiting: &mut InternedPathSet,
  ) -> Result<Option<ContextValue>> {
    let symlink_metadata = match self.inner.fs.symlink_metadata(path.assert_utf8()).await {
      Ok(metadata) => Some(metadata),
      Err(error) if is_not_found(&error) => None,
      Err(error) => return Err(error.into()),
    };
    let Some(symlink_metadata) = symlink_metadata else {
      return Ok(None);
    };

    if symlink_metadata.is_symlink {
      let target = self.inner.fs.read_link(path.assert_utf8()).await?;
      let target = if target.node_is_absolute() {
        target
      } else {
        path
          .assert_utf8()
          .parent()
          .expect("symlink path should have a parent")
          .node_join(&target)
      };
      let target = InternedPath::from(target.node_normalize());
      let mut value = self.context_leaf(target.to_string_lossy().as_bytes(), mode, 0);
      if let Some(target_value) = self.context_value(&target, mode, visiting).await? {
        value.safe_time = value.safe_time.max(target_value.safe_time);
        value.timestamp_hash = merge_digests(
          self.inner.hash_function,
          value.timestamp_hash,
          target_value.timestamp_hash,
        );
        value.hash = merge_digests(self.inner.hash_function, value.hash, target_value.hash);
      }
      return Ok(Some(value));
    }

    let Some(metadata) = self.metadata(path).await? else {
      return Ok(None);
    };
    if !metadata.is_directory {
      return self.file_context_value(path, mode).await;
    }

    let mut children = match self.inner.fs.read_dir(path.assert_utf8()).await {
      Ok(children) => children,
      Err(error) if is_not_found(&error) => return Ok(None),
      Err(error) => return Err(error.into()),
    };
    children.sort_unstable();

    let mut timestamp_hasher = mode
      .uses_timestamp()
      .then(|| RspackHasher::new(&self.inner.hash_function));
    let mut content_hasher = mode
      .uses_hash()
      .then(|| RspackHasher::new(&self.inner.hash_function));
    for child in &children {
      if let Some(hasher) = &mut timestamp_hasher {
        hasher.write(child.as_bytes());
      }
      if let Some(hasher) = &mut content_hasher {
        hasher.write(child.as_bytes());
      }
    }

    let mut safe_time = 0;
    for child in children {
      let child_path = InternedPath::from(path.join(child));
      let child_value = match self.check_managed(&child_path) {
        PathClassification::Immutable => None,
        PathClassification::Managed(managed_item) => {
          match self.get_managed_item_info(&managed_item).await? {
            Some(info) => Some(self.context_leaf(info.as_bytes(), mode, 0)),
            None => self.context_value(&child_path, mode, visiting).await?,
          }
        }
        PathClassification::Unmanaged => self.context_value(&child_path, mode, visiting).await?,
      };

      let Some(child_value) = child_value else {
        if let Some(hasher) = &mut timestamp_hasher {
          hasher.write(b"n");
        }
        continue;
      };
      safe_time = safe_time.max(child_value.safe_time);
      if let Some(hasher) = &mut timestamp_hasher {
        if let Some(hash) = child_value.timestamp_hash {
          hasher.write(b"d");
          hasher.write(hash.encoded().as_bytes());
        } else {
          hasher.write(b"n");
        }
      }
      if let Some(hasher) = &mut content_hasher
        && let Some(hash) = child_value.hash
      {
        hasher.write(hash.encoded().as_bytes());
      }
    }

    Ok(Some(ContextValue {
      safe_time,
      timestamp_hash: timestamp_hasher.map(digest_hasher),
      hash: content_hasher.map(digest_hasher),
    }))
  }

  async fn file_context_value(
    &self,
    path: &InternedPath,
    mode: SnapshotMode,
  ) -> Result<Option<ContextValue>> {
    let timestamp = if mode.uses_timestamp() {
      self.file_timestamp(path).await?
    } else {
      None
    };
    let hash = if mode.uses_hash() {
      self.file_hash(path).await?
    } else {
      None
    };
    if timestamp.is_none() && hash.is_none() {
      return Ok(None);
    }

    let safe_time = timestamp.as_ref().map_or(0, |entry| entry.safe_time);
    let timestamp_hash = timestamp.map(|entry| {
      let mut hasher = RspackHasher::new(&self.inner.hash_function);
      if let Some(timestamp) = entry.timestamp {
        hasher.write(b"f");
        hasher.write(timestamp.to_string().as_bytes());
      }
      digest_hasher(hasher)
    });
    let hash = hash.map(|hash| match hash {
      FileHash::Digest(hash) => hash,
      FileHash::Directory => self.digest(b"directory"),
    });
    Ok(Some(ContextValue {
      safe_time,
      timestamp_hash,
      hash,
    }))
  }

  fn context_leaf(&self, bytes: &[u8], mode: SnapshotMode, safe_time: u64) -> ContextValue {
    ContextValue {
      safe_time,
      timestamp_hash: mode.uses_timestamp().then(|| self.digest(bytes)),
      hash: mode.uses_hash().then(|| self.digest(bytes)),
    }
  }

  fn digest(&self, bytes: &[u8]) -> RspackHashDigest {
    let mut hasher = RspackHasher::new(&self.inner.hash_function);
    hasher.write(bytes);
    digest_hasher(hasher)
  }

  /// Corresponds to webpack's `FileSystemInfo._getManagedItemDirectoryInfo`.
  /// The cache serves the same purpose as webpack's `managedItemDirectoryQueue`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L4488-L4502
  async fn get_managed_item_directory_info(
    &self,
    path: &InternedPath,
  ) -> Result<Arc<InternedPathSet>> {
    if let Some(elements) = self.inner.managed_item_directory_info.get(path) {
      return Ok(Arc::clone(&elements));
    }
    let elements = match self.inner.fs.read_dir(path.assert_utf8()).await {
      Ok(elements) => elements
        .into_iter()
        .map(|element| InternedPath::from(path.join(element)))
        .collect(),
      Err(error) if is_not_found_or_not_a_directory(&error) => InternedPathSet::default(),
      Err(error) => return Err(error.into()),
    };
    let elements = Arc::new(elements);
    self
      .inner
      .managed_item_directory_info
      .insert(path.clone(), Arc::clone(&elements));
    Ok(elements)
  }

  /// Corresponds to webpack's `FileSystemInfo._getManagedItemInfo`.
  /// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L4508-L4576
  async fn get_managed_item_info(&self, path: &InternedPath) -> Result<Option<String>> {
    if let Some(info) = self.inner.managed_items.get(path) {
      return Ok(info.clone());
    }
    let dir = InternedPath::from(path.parent().unwrap_or(path));
    let elements = self.get_managed_item_directory_info(&dir).await?;
    if !elements.contains(path) {
      let info = "*missing".to_string();
      self
        .inner
        .managed_items
        .insert(path.clone(), Some(info.clone()));
      return Ok(Some(info));
    }
    if path.file_name().is_some_and(|name| name == "node_modules") {
      let info = "*node_modules".to_string();
      self
        .inner
        .managed_items
        .insert(path.clone(), Some(info.clone()));
      return Ok(Some(info));
    }
    let package_json_path = InternedPath::from(path.join("package.json"));
    let mut content = match self.inner.fs.read(package_json_path.assert_utf8()).await {
      Ok(content) => content,
      Err(error) if is_not_found_or_not_a_directory(&error) => {
        if let Ok(elements) = self.inner.fs.read_dir(path.assert_utf8()).await
          && elements.len() == 1
          && elements[0] == "node_modules"
        {
          let info = "*nested".to_string();
          self
            .inner
            .managed_items
            .insert(path.clone(), Some(info.clone()));
          return Ok(Some(info));
        }
        self.inner.logger.warn(format!(
          "Managed item {} isn't a directory or doesn't contain a package.json (see snapshot.managedPaths option)",
          path.display()
        ));
        self.inner.managed_items.insert(path.clone(), None);
        return Ok(None);
      }
      Err(error) => return Err(error.into()),
    };
    let data = simd_json::to_borrowed_value(&mut content)
      .map_err(|error| error!("Failed to parse {package_json_path:?}: {error}"))?;
    let Some(name) = data
      .get("name")
      .and_then(|value| value.as_str())
      .filter(|name| !name.is_empty())
    else {
      self.inner.logger.warn(format!(
        "{} doesn't contain a \"name\" property (see snapshot.managedPaths option)",
        package_json_path.display()
      ));
      self.inner.managed_items.insert(path.clone(), None);
      return Ok(None);
    };
    let version = data
      .get("version")
      .and_then(|value| value.as_str())
      .unwrap_or_default();
    let info = format!("{name}@{version}");
    self
      .inner
      .managed_items
      .insert(path.clone(), Some(info.clone()));
    Ok(Some(info))
  }

  #[async_recursion::async_recursion]
  async fn validate_snapshot(
    &self,
    snapshot: &Snapshot,
    modified_files: &mut InternedPathSet,
    removed_files: &mut InternedPathSet,
  ) -> Result<()> {
    if let Some(children) = &snapshot.children {
      for child in children {
        self
          .validate_snapshot(child, modified_files, removed_files)
          .await?;
      }
    }

    if let Some(entries) = &snapshot.file_timestamps {
      for (path, expected) in entries {
        let current = self.file_timestamp(path).await?;
        if !file_timestamp_matches(current.as_ref(), expected.as_ref(), snapshot.start_time) {
          record_invalid(path, current.is_some(), modified_files, removed_files);
        }
      }
    }
    if let Some(entries) = &snapshot.file_hashes {
      for (path, expected) in entries {
        let current = self.file_hash(path).await?;
        if current != *expected {
          record_invalid(path, current.is_some(), modified_files, removed_files);
        }
      }
    }
    if let Some(entries) = &snapshot.file_timestamp_hashes {
      for (path, expected) in entries {
        let current_timestamp = self.file_timestamp(path).await?;
        let timestamp_matches = match expected {
          Some(expected) => file_timestamp_matches(
            current_timestamp.as_ref(),
            Some(&FileSystemInfoEntry {
              safe_time: expected.safe_time,
              timestamp: expected.timestamp,
            }),
            snapshot.start_time,
          ),
          None => current_timestamp.is_none(),
        };
        if timestamp_matches {
          continue;
        }
        let current_hash = self.file_hash(path).await?;
        let expected_hash = expected.as_ref().map(|entry| &entry.hash);
        if current_hash.as_ref() != expected_hash {
          record_invalid(path, current_hash.is_some(), modified_files, removed_files);
        }
      }
    }

    if let Some(entries) = &snapshot.context_timestamps {
      for (path, expected) in entries {
        let current = self.context_timestamp(path).await?;
        if !context_timestamp_matches(current.as_ref(), expected.as_ref(), snapshot.start_time) {
          record_invalid(path, current.is_some(), modified_files, removed_files);
        }
      }
    }
    if let Some(entries) = &snapshot.context_hashes {
      for (path, expected) in entries {
        let current = self.context_hash(path).await?;
        if current != *expected {
          record_invalid(path, current.is_some(), modified_files, removed_files);
        }
      }
    }
    if let Some(entries) = &snapshot.context_timestamp_hashes {
      for (path, expected) in entries {
        let current_timestamp = self.context_timestamp(path).await?;
        let timestamp_matches = match expected {
          Some(expected) => context_timestamp_matches(
            current_timestamp.as_ref(),
            Some(&ContextFileSystemInfoEntry {
              safe_time: expected.safe_time,
              timestamp_hash: expected.timestamp_hash.clone(),
            }),
            snapshot.start_time,
          ),
          None => current_timestamp.is_none(),
        };
        if timestamp_matches {
          continue;
        }
        let current_hash = self.context_hash(path).await?;
        let expected_hash = expected.as_ref().map(|entry| &entry.hash);
        if current_hash.as_ref() != expected_hash {
          record_invalid(path, current_hash.is_some(), modified_files, removed_files);
        }
      }
    }

    if let Some(entries) = &snapshot.missing_existence {
      for (path, expected) in entries {
        let current = self.metadata(path).await?.is_some();
        if current != *expected {
          record_invalid(path, current, modified_files, removed_files);
        }
      }
    }
    if let Some(entries) = &snapshot.managed_item_info {
      for (path, expected) in entries {
        let current = self.get_managed_item_info(path).await?;
        if current.as_ref() != Some(expected) {
          record_invalid(path, current.is_some(), modified_files, removed_files);
        }
      }
    }
    Ok(())
  }
}

impl SnapshotMode {
  fn uses_timestamp(self) -> bool {
    !matches!(self, Self::Hash)
  }

  fn uses_hash(self) -> bool {
    !matches!(self, Self::Timestamp)
  }
}

/// Corresponds to webpack's constructor initialization of `*PathsWithSlash`
/// (`join(fs, path, "_").slice(0, -1)`) and `*PathsRegExps`.
/// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L1448-L1476
fn split_snapshot_paths(paths: Vec<PathMatcher>) -> (Vec<String>, Vec<RspackRegex>) {
  let mut paths_with_slash = Vec::new();
  let mut paths_reg_exps = Vec::new();
  for path in paths {
    match path {
      PathMatcher::String(path) => {
        let path = Utf8Path::new(&path);
        let path = if path.node_is_absolute_posix() {
          path.node_join_posix("_").node_normalize_posix()
        } else if path.node_is_absolute_win32() {
          path.node_join_win32("_").node_normalize_win32()
        } else {
          path.node_join("_").node_normalize()
        };
        let mut path = path.into_string();
        path.pop();
        paths_with_slash.push(path);
      }
      PathMatcher::Regexp(regex) => paths_reg_exps.push(regex),
    }
  }
  (paths_with_slash, paths_reg_exps)
}

/// Corresponds to the top-level `getManagedItem` in webpack's `FileSystemInfo.js`.
/// https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/lib/FileSystemInfo.js#L1148-L1207
fn get_managed_item<'a>(managed_path: &str, path: &'a str) -> Option<&'a str> {
  let bytes = path.as_bytes();
  let mut i = managed_path.len();
  let mut slashes = 1;
  let mut starting_position = true;
  while i < bytes.len() {
    match bytes[i] {
      b'/' | b'\\' => {
        slashes -= 1;
        if slashes == 0 {
          break;
        }
        starting_position = true;
      }
      b'.' => {
        if starting_position {
          return None;
        }
      }
      b'@' => {
        if !starting_position {
          return None;
        }
        slashes += 1;
      }
      _ => starting_position = false,
    }
    i += 1;
  }
  if i == bytes.len() {
    slashes -= 1;
  }
  if slashes != 0 {
    return None;
  }
  if let Some(rest) = path.get(i + 1..)
    && let Some(rest) = rest.strip_prefix("node_modules")
  {
    if rest.is_empty() {
      return Some(path);
    }
    if rest.starts_with(['/', '\\']) {
      return get_managed_item(&path[..i + "/node_modules/".len()], path);
    }
  }
  path.get(..i)
}

fn file_timestamp_matches(
  current: Option<&FileSystemInfoEntry>,
  expected: Option<&FileSystemInfoEntry>,
  start_time: Option<u64>,
) -> bool {
  match (current, expected) {
    (None, None) => true,
    (Some(current), Some(expected)) => {
      if start_time.is_some_and(|start_time| current.safe_time > start_time) {
        return false;
      }
      current.timestamp == expected.timestamp
    }
    _ => false,
  }
}

fn context_timestamp_matches(
  current: Option<&ContextFileSystemInfoEntry>,
  expected: Option<&ContextFileSystemInfoEntry>,
  start_time: Option<u64>,
) -> bool {
  match (current, expected) {
    (None, None) => true,
    (Some(current), Some(expected)) => {
      if start_time.is_some_and(|start_time| current.safe_time > start_time) {
        return false;
      }
      current.timestamp_hash == expected.timestamp_hash
    }
    _ => false,
  }
}

fn record_invalid(
  path: &InternedPath,
  currently_exists: bool,
  modified_files: &mut InternedPathSet,
  removed_files: &mut InternedPathSet,
) {
  if currently_exists {
    modified_files.insert(path.clone());
  } else {
    removed_files.insert(path.clone());
  }
}

fn merge_digests(
  hash_function: HashFunction,
  first: Option<RspackHashDigest>,
  second: Option<RspackHashDigest>,
) -> Option<RspackHashDigest> {
  match (first, second) {
    (None, None) => None,
    (first, second) => {
      let mut hasher = RspackHasher::new(&hash_function);
      if let Some(first) = first {
        hasher.write(first.encoded().as_bytes());
      }
      if let Some(second) = second {
        hasher.write(second.encoded().as_bytes());
      }
      Some(digest_hasher(hasher))
    }
  }
}

fn digest_hasher(hasher: RspackHasher) -> RspackHashDigest {
  hasher.digest(&HashDigest::Hex)
}

fn is_not_found(error: &FsError) -> bool {
  matches!(error, FsError::Io(error) if error.kind() == std::io::ErrorKind::NotFound)
}

fn is_not_found_or_not_a_directory(error: &FsError) -> bool {
  matches!(error, FsError::Io(error) if matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory))
}

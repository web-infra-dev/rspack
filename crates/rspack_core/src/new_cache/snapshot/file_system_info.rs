use std::{collections::VecDeque, fmt, sync::Arc};

use rspack_error::{Result, error};
use rspack_fs::{Error as FsError, FileMetadata, ReadableFileSystem};
use rspack_hash::{HashDigest, HashFunction, RspackHashDigest, RspackHasher};
use rspack_parallel::TryFutureConsumer;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathDashMap, InternedPathSet};
use rspack_util::time::mtime_accuracy;
use simd_json::prelude::{ValueAsScalar, ValueObjectAccess};

use super::{
  ContextFileSystemInfoEntry, ContextTimestampAndHash, FileHash, FileSystemInfoEntry, Snapshot,
  TimestampAndHash,
};
use crate::{
  CompilationLogger, InfrastructureLogger, LogType, Logger,
  cache::{BuildDependencyHelper, SnapshotOptions, SnapshotStrategyOptions, is_node_package_path},
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

#[derive(Debug, Clone, Copy)]
enum PathKind {
  File,
  Context,
  Missing,
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
  options: SnapshotOptions,
  hash_function: HashFunction,
  file_timestamps: InternedPathDashMap<Option<FileSystemInfoEntry>>,
  file_hashes: InternedPathDashMap<Option<FileHash>>,
  file_timestamp_hashes: InternedPathDashMap<Option<TimestampAndHash>>,
  context_timestamps: InternedPathDashMap<Option<ContextFileSystemInfoEntry>>,
  context_hashes: InternedPathDashMap<Option<RspackHashDigest>>,
  context_timestamp_hashes: InternedPathDashMap<Option<ContextTimestampAndHash>>,
  managed_items: InternedPathDashMap<Option<String>>,
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
    Self {
      inner: Arc::new(FileSystemInfoInner {
        fs,
        logger: logger.into(),
        options,
        hash_function,
        file_timestamps: Default::default(),
        file_hashes: Default::default(),
        file_timestamp_hashes: Default::default(),
        context_timestamps: Default::default(),
        context_hashes: Default::default(),
        context_timestamp_hashes: Default::default(),
        managed_items: Default::default(),
      }),
    }
  }

  /// See webpack's snapshot creation implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L2525-L3079
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

    let files = self
      .capture_non_managed(&mut snapshot, files, PathKind::File)
      .await?;
    let contexts = self
      .capture_non_managed(&mut snapshot, contexts, PathKind::Context)
      .await?;
    let missing = self
      .capture_non_managed(&mut snapshot, missing, PathKind::Missing)
      .await?;

    self.capture_files(&mut snapshot, files, mode).await?;
    self.capture_contexts(&mut snapshot, contexts, mode).await?;
    self.capture_missing(&mut snapshot, missing).await?;
    Ok(snapshot)
  }

  /// See webpack's snapshot merge implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L3081-L3166
  pub fn merge_snapshots(&self, mut first: Snapshot, second: Snapshot) -> Snapshot {
    first.merge(second);
    first
  }

  pub fn build_dependencies_strategy(&self) -> SnapshotStrategyOptions {
    self.inner.options.dependencies_strategy()
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

  async fn capture_non_managed(
    &self,
    snapshot: &mut Snapshot,
    paths: &InternedPathSet,
    kind: PathKind,
  ) -> Result<Vec<InternedPath>> {
    let mut captured = Vec::with_capacity(paths.len());
    for path in paths {
      let path_str = path.to_string_lossy();
      if self.inner.options.is_immutable_path(&path_str) {
        managed_paths(snapshot, kind).insert(path.clone());
        continue;
      }
      if self.inner.options.is_managed_path(&path_str)
        && let Some((managed_item, info)) = self.find_managed_item(path).await?
      {
        managed_paths(snapshot, kind).insert(path.clone());
        snapshot
          .managed_files
          .get_or_insert_default()
          .insert(InternedPath::from(managed_item.join("package.json")));
        snapshot
          .managed_item_info
          .get_or_insert_default()
          .insert(managed_item, info);
        continue;
      }
      captured.push(path.clone());
    }
    Ok(captured)
  }

  async fn capture_files(
    &self,
    snapshot: &mut Snapshot,
    paths: Vec<InternedPath>,
    mode: SnapshotMode,
  ) -> Result<()> {
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

  async fn capture_contexts(
    &self,
    snapshot: &mut Snapshot,
    paths: Vec<InternedPath>,
    mode: SnapshotMode,
  ) -> Result<()> {
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

  async fn capture_missing(&self, snapshot: &mut Snapshot, paths: Vec<InternedPath>) -> Result<()> {
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

    let result = self.context_value_inner(path, mode, visiting).await;
    visiting.remove(path);
    result
  }

  async fn context_value_inner(
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
      let target = self.inner.fs.canonicalize(path.assert_utf8()).await?;
      let target = InternedPath::from(target);
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

    let metadata = self
      .metadata(path)
      .await?
      .expect("symlink metadata proved that the path exists");
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
      let child_path_str = child_path.to_string_lossy();
      let child_value = if self.inner.options.is_immutable_path(&child_path_str) {
        None
      } else if self.inner.options.is_managed_path(&child_path_str) {
        self
          .find_managed_item(&child_path)
          .await?
          .map(|(_, info)| self.context_leaf(info.as_bytes(), mode, 0))
      } else {
        self.context_value(&child_path, mode, visiting).await?
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

  async fn find_managed_item(&self, path: &InternedPath) -> Result<Option<(InternedPath, String)>> {
    let mut current = if self
      .metadata(path)
      .await?
      .is_some_and(|metadata| metadata.is_directory)
    {
      Some(path.clone())
    } else {
      path.parent().map(InternedPath::from)
    };
    while let Some(directory) = current {
      let directory_str = directory.to_string_lossy();
      if !self.inner.options.is_managed_path(&directory_str) {
        break;
      }
      if let Some(info) = self.managed_item_info(&directory).await? {
        return Ok(Some((directory, info)));
      }
      current = directory.parent().map(InternedPath::from);
    }
    Ok(None)
  }

  /// See webpack's managed item metadata implementation:
  /// https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FileSystemInfo.js#L4505-L4577
  async fn managed_item_info(&self, path: &InternedPath) -> Result<Option<String>> {
    if let Some(info) = self.inner.managed_items.get(path) {
      return Ok(info.clone());
    }
    let package_json = InternedPath::from(path.join("package.json"));
    let mut content = match self.inner.fs.read(package_json.assert_utf8()).await {
      Ok(content) => content,
      Err(error) if is_not_found(&error) => {
        self.inner.managed_items.insert(path.clone(), None);
        return Ok(None);
      }
      Err(error) => return Err(error.into()),
    };
    let package_json = simd_json::to_borrowed_value(&mut content)
      .map_err(|error| error!("Failed to parse {package_json:?}: {error}"))?;
    let Some(name) = package_json.get("name").and_then(|value| value.as_str()) else {
      self.inner.managed_items.insert(path.clone(), None);
      return Ok(None);
    };
    let version = package_json
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
        let current = self.managed_item_info(path).await?;
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

fn managed_paths(snapshot: &mut Snapshot, kind: PathKind) -> &mut InternedPathSet {
  match kind {
    PathKind::File => snapshot.managed_files.get_or_insert_default(),
    PathKind::Context => snapshot.managed_contexts.get_or_insert_default(),
    PathKind::Missing => snapshot.managed_missing.get_or_insert_default(),
  }
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

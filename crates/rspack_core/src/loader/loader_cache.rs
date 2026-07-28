use std::{
  fs::{self, OpenOptions},
  io::{ErrorKind, Write},
  path::{Path, PathBuf},
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{AdditionalData, Content, Loader, LoaderContext, Scheme};
use rspack_paths::Utf8PathBuf;
use rspack_sources::SourceMap;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::{
  ApplyContext, BoxLoader, ModuleRuleUseLoader, NormalModuleFactoryCreateLoaderCache, Plugin,
  RunnerContext,
};

pub(crate) const INTERNAL_CACHE_LOADER_IDENTIFIER: &str = "builtin:cache-loader";

const FORMAT_VERSION: u8 = 1;
const FUTURE_TIMESTAMP_TOLERANCE: Duration = Duration::from_secs(2);
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_millis(500);
const STALE_LOCK_AGE: Duration = Duration::from_secs(30);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct LoaderCacheKey {
  rspack_version: String,
  module_identifier: String,
  remaining_request: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct ResourceStamp {
  mtime_ns: u64,
  size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DependencyDelta {
  added: FxHashSet<PathBuf>,
  removed: FxHashSet<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DependencySnapshot {
  path: String,
  kind: u8,
  fingerprint: u64,
}

#[derive(Debug, Clone)]
struct LoaderCacheEntry {
  resource: ResourceStamp,
  written_at_ns: u64,
  content: Option<Content>,
  source_map: Option<String>,
  additional_data: Option<AdditionalData>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  dependency_snapshot: Vec<DependencySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PersistedContent {
  String(String),
  Buffer(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedPayload {
  identity: LoaderCacheKey,
  resource: ResourceStamp,
  written_at_ns: u64,
  content: PersistedContent,
  source_map: Option<String>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  dependency_snapshot: Vec<DependencySnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedEnvelope {
  version: u8,
  checksum: u64,
  payload: PersistedPayload,
}

#[derive(Debug, Serialize, Deserialize)]
struct PitchData {
  key: LoaderCacheKey,
  digest: String,
  resource: ResourceStamp,
  diagnostics_len: usize,
  file_dependencies: FxHashSet<PathBuf>,
  context_dependencies: FxHashSet<PathBuf>,
  missing_dependencies: FxHashSet<PathBuf>,
  build_dependencies: FxHashSet<PathBuf>,
}

/// A deliberately small loader-cache file backend.
///
/// It owns only byte IO and locking. Cache identity, validation and replay stay
/// in `LoaderCacheService`.
#[derive(Debug, Clone)]
pub(crate) struct LoaderCacheFileStore {
  root: Utf8PathBuf,
  readonly: bool,
}

impl LoaderCacheFileStore {
  pub(crate) fn new(root: Utf8PathBuf, readonly: bool) -> Self {
    Self { root, readonly }
  }

  fn entry_path(&self, digest: &str) -> PathBuf {
    self.root.join(format!("{digest}.json")).into_std_path_buf()
  }

  fn lock_path(&self, digest: &str) -> PathBuf {
    self.root.join(format!("{digest}.lock")).into_std_path_buf()
  }

  async fn get(&self, digest: &str) -> Option<Vec<u8>> {
    let path = self.entry_path(digest);
    tokio::task::spawn_blocking(move || fs::read(path))
      .await
      .ok()?
      .ok()
  }

  async fn put(&self, digest: &str, value: Vec<u8>) {
    if self.readonly {
      return;
    }
    let entry_path = self.entry_path(digest);
    let lock_path = self.lock_path(digest);
    let _ =
      tokio::task::spawn_blocking(move || write_atomic_with_lock(&entry_path, &lock_path, &value))
        .await;
  }

  async fn remove(&self, digest: &str) {
    if self.readonly {
      return;
    }
    let entry_path = self.entry_path(digest);
    let lock_path = self.lock_path(digest);
    let _ =
      tokio::task::spawn_blocking(move || remove_with_lock(&entry_path, &lock_path, None)).await;
  }

  async fn remove_if_unchanged(&self, digest: &str, expected: Vec<u8>) {
    if self.readonly {
      return;
    }
    let entry_path = self.entry_path(digest);
    let lock_path = self.lock_path(digest);
    let _ = tokio::task::spawn_blocking(move || {
      remove_with_lock(&entry_path, &lock_path, Some(&expected))
    })
    .await;
  }
}

#[derive(Debug)]
struct FileLock {
  path: PathBuf,
}

impl Drop for FileLock {
  fn drop(&mut self) {
    let _ = fs::remove_file(&self.path);
  }
}

fn acquire_file_lock(path: &Path) -> std::io::Result<FileLock> {
  let start = Instant::now();
  loop {
    match OpenOptions::new().write(true).create_new(true).open(path) {
      Ok(mut file) => {
        let _ = writeln!(file, "{}", std::process::id());
        return Ok(FileLock {
          path: path.to_path_buf(),
        });
      }
      Err(error) if error.kind() == ErrorKind::AlreadyExists => {
        let stale = fs::metadata(path)
          .and_then(|metadata| metadata.modified())
          .and_then(|modified| {
            SystemTime::now()
              .duration_since(modified)
              .map_err(std::io::Error::other)
          })
          .is_ok_and(|age| age >= STALE_LOCK_AGE);
        if stale {
          let _ = fs::remove_file(path);
          continue;
        }
        if start.elapsed() >= LOCK_WAIT_TIMEOUT {
          return Err(std::io::Error::new(
            ErrorKind::WouldBlock,
            "timed out waiting for loader cache lock",
          ));
        }
        std::thread::sleep(LOCK_RETRY_INTERVAL);
      }
      Err(error) => return Err(error),
    }
  }
}

fn write_atomic_with_lock(
  entry_path: &Path,
  lock_path: &Path,
  value: &[u8],
) -> std::io::Result<()> {
  let Some(parent) = entry_path.parent() else {
    return Err(std::io::Error::other(
      "loader cache entry has no parent directory",
    ));
  };
  fs::create_dir_all(parent)?;
  let _lock = acquire_file_lock(lock_path)?;
  let temp_id = TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
  let temp_path = entry_path.with_extension(format!("json.tmp.{}.{}", std::process::id(), temp_id));
  let result = (|| {
    let mut file = OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(&temp_path)?;
    file.write_all(value)?;
    file.sync_all()?;
    drop(file);
    match fs::rename(&temp_path, entry_path) {
      Ok(()) => Ok(()),
      Err(error)
        if cfg!(windows)
          && matches!(
            error.kind(),
            ErrorKind::AlreadyExists | ErrorKind::PermissionDenied
          ) =>
      {
        let _ = fs::remove_file(entry_path);
        fs::rename(&temp_path, entry_path)
      }
      Err(error) => Err(error),
    }
  })();
  if result.is_err() {
    let _ = fs::remove_file(temp_path);
  }
  result
}

fn remove_with_lock(
  entry_path: &Path,
  lock_path: &Path,
  expected: Option<&[u8]>,
) -> std::io::Result<()> {
  if !entry_path.exists() {
    return Ok(());
  }
  let _lock = acquire_file_lock(lock_path)?;
  if let Some(expected) = expected
    && fs::read(entry_path).ok().as_deref() != Some(expected)
  {
    return Ok(());
  }
  match fs::remove_file(entry_path) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error),
  }
}

#[derive(Debug)]
pub(crate) struct LoaderCacheService {
  entries: FxDashMap<LoaderCacheKey, LoaderCacheEntry>,
  file_store: Option<LoaderCacheFileStore>,
}

impl LoaderCacheService {
  pub(crate) fn memory_only() -> Self {
    Self::new(None)
  }

  pub(crate) fn new(file_store: Option<LoaderCacheFileStore>) -> Self {
    Self {
      entries: FxDashMap::default(),
      file_store,
    }
  }

  async fn lookup(
    &self,
    key: &LoaderCacheKey,
    digest: &str,
    resource: ResourceStamp,
    fs: &dyn ReadableFileSystem,
  ) -> Option<LoaderCacheEntry> {
    let now_ns = now_ns()?;
    if let Some(entry) = self.entries.get(key).map(|entry| entry.value().clone())
      && resource_is_valid(&entry, resource, now_ns)
    {
      return Some(entry);
    }
    self.entries.remove(key);

    let file_store = self.file_store.as_ref()?;
    let bytes = file_store.get(digest).await?;
    let entry = match decode_entry(&bytes, key) {
      Ok(entry) => entry,
      Err(()) => {
        file_store.remove_if_unchanged(digest, bytes).await;
        return None;
      }
    };
    if !entry_is_valid(&entry, resource, now_ns, fs).await {
      file_store.remove_if_unchanged(digest, bytes).await;
      return None;
    }
    self.entries.insert(key.clone(), entry.clone());
    Some(entry)
  }

  async fn store(
    &self,
    key: LoaderCacheKey,
    digest: String,
    entry: LoaderCacheEntry,
    fs: &dyn ReadableFileSystem,
  ) {
    if !validate_dependencies(&entry.dependency_snapshot, fs).await {
      return;
    }
    self.entries.insert(key.clone(), entry.clone());

    let Some(file_store) = &self.file_store else {
      return;
    };
    // AdditionalData has no stable serialization contract. This entry remains
    // useful in L1, but any older disk entry must not be reused.
    if entry.additional_data.is_some() {
      file_store.remove(&digest).await;
      return;
    }
    let Some(bytes) = encode_entry(key, entry) else {
      return;
    };
    file_store.put(&digest, bytes).await;
  }

  async fn remove(&self, key: &LoaderCacheKey, digest: &str) {
    self.entries.remove(key);
    if let Some(file_store) = &self.file_store {
      file_store.remove(digest).await;
    }
  }
}

fn encode_entry(key: LoaderCacheKey, entry: LoaderCacheEntry) -> Option<Vec<u8>> {
  let content = match entry.content? {
    Content::String(value) => PersistedContent::String(value),
    Content::Buffer(value) => PersistedContent::Buffer(value),
  };
  let payload = PersistedPayload {
    identity: key,
    resource: entry.resource,
    written_at_ns: entry.written_at_ns,
    content,
    source_map: entry.source_map,
    file_dependencies: entry.file_dependencies,
    context_dependencies: entry.context_dependencies,
    missing_dependencies: entry.missing_dependencies,
    build_dependencies: entry.build_dependencies,
    dependency_snapshot: entry.dependency_snapshot,
  };
  let payload_bytes = serde_json::to_vec(&payload).ok()?;
  serde_json::to_vec(&PersistedEnvelope {
    version: FORMAT_VERSION,
    checksum: checksum(&payload_bytes),
    payload,
  })
  .ok()
}

fn decode_entry(
  bytes: &[u8],
  expected_key: &LoaderCacheKey,
) -> std::result::Result<LoaderCacheEntry, ()> {
  let envelope: PersistedEnvelope = serde_json::from_slice(bytes).map_err(|_| ())?;
  if envelope.version != FORMAT_VERSION || &envelope.payload.identity != expected_key {
    return Err(());
  }
  let payload_bytes = serde_json::to_vec(&envelope.payload).map_err(|_| ())?;
  if checksum(&payload_bytes) != envelope.checksum {
    return Err(());
  }
  let content = match envelope.payload.content {
    PersistedContent::String(value) => Content::String(value),
    PersistedContent::Buffer(value) => Content::Buffer(value),
  };
  Ok(LoaderCacheEntry {
    resource: envelope.payload.resource,
    written_at_ns: envelope.payload.written_at_ns,
    content: Some(content),
    source_map: envelope.payload.source_map,
    additional_data: None,
    file_dependencies: envelope.payload.file_dependencies,
    context_dependencies: envelope.payload.context_dependencies,
    missing_dependencies: envelope.payload.missing_dependencies,
    build_dependencies: envelope.payload.build_dependencies,
    dependency_snapshot: envelope.payload.dependency_snapshot,
  })
}

async fn entry_is_valid(
  entry: &LoaderCacheEntry,
  resource: ResourceStamp,
  now_ns: u64,
  fs: &dyn ReadableFileSystem,
) -> bool {
  resource_is_valid(entry, resource, now_ns)
    && validate_dependencies(&entry.dependency_snapshot, fs).await
}

fn resource_is_valid(entry: &LoaderCacheEntry, resource: ResourceStamp, now_ns: u64) -> bool {
  entry.resource == resource
    && timestamp_not_in_future(entry.resource.mtime_ns, now_ns)
    && timestamp_not_in_future(entry.written_at_ns, now_ns)
}

fn timestamp_not_in_future(timestamp_ns: u64, now_ns: u64) -> bool {
  timestamp_ns
    <= now_ns.saturating_add(
      FUTURE_TIMESTAMP_TOLERANCE
        .as_nanos()
        .try_into()
        .unwrap_or(u64::MAX),
    )
}

fn now_ns() -> Option<u64> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .ok()?
    .as_nanos()
    .try_into()
    .ok()
}

fn checksum(value: &[u8]) -> u64 {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  hasher.write(value);
  hasher.finish()
}

fn cache_key(loader_context: &LoaderContext<RunnerContext>) -> (LoaderCacheKey, String) {
  let key = LoaderCacheKey {
    rspack_version: env!("CARGO_PKG_VERSION").to_string(),
    module_identifier: loader_context
      .context
      .module
      .identifier()
      .as_str()
      .to_owned(),
    remaining_request: loader_context.remaining_request().to_string(),
  };
  let bytes = serde_json::to_vec(&key).expect("loader cache key should be serializable");
  let digest = format!("{:016x}", checksum(&bytes));
  (key, digest)
}

async fn resource_stamp(loader_context: &LoaderContext<RunnerContext>) -> Option<ResourceStamp> {
  if loader_context.resource_data().get_scheme() != &Scheme::None {
    return None;
  }
  let path = loader_context.resource_path()?;
  if path.as_str().is_empty() {
    return None;
  }
  let metadata = loader_context.context.fs.metadata(path).await.ok()?;
  if !metadata.is_file {
    return None;
  }
  let mtime_ns = metadata.mtime_ms.saturating_mul(1_000_000);
  if !timestamp_not_in_future(mtime_ns, now_ns()?) {
    return None;
  }
  Some(ResourceStamp {
    mtime_ns,
    size: metadata.size,
  })
}

async fn dependency_fingerprint(path: &str, kind: u8, fs: &dyn ReadableFileSystem) -> Option<u64> {
  let path = rspack_paths::Utf8Path::new(path);
  if kind == 2 {
    let parent = path.parent()?;
    let mut entries = fs.read_dir(parent).await.ok()?;
    entries.sort();
    return Some(checksum(entries.join("\0").as_bytes()));
  }
  let metadata = fs.metadata(path).await.ok()?;
  if kind == 1 || metadata.is_directory {
    let mut entries = fs.read_dir(path).await.ok()?;
    entries.sort();
    return Some(checksum(entries.join("\0").as_bytes()));
  }
  Some(checksum(&fs.read(path).await.ok()?))
}

async fn validate_dependencies(
  dependencies: &[DependencySnapshot],
  fs: &dyn ReadableFileSystem,
) -> bool {
  for dependency in dependencies {
    if dependency_fingerprint(&dependency.path, dependency.kind, fs).await
      != Some(dependency.fingerprint)
    {
      return false;
    }
  }
  true
}

async fn dependency_snapshots(
  file: &FxHashSet<PathBuf>,
  context: &FxHashSet<PathBuf>,
  missing: &FxHashSet<PathBuf>,
  build: &FxHashSet<PathBuf>,
  fs: &dyn ReadableFileSystem,
) -> Option<Vec<DependencySnapshot>> {
  let mut snapshots = Vec::new();
  for (kind, values) in [(0, file), (1, context), (2, missing), (0, build)] {
    for path in values {
      let path = path.to_string_lossy().into_owned();
      snapshots.push(DependencySnapshot {
        fingerprint: dependency_fingerprint(&path, kind, fs).await?,
        path,
        kind,
      });
    }
  }
  Some(snapshots)
}

fn dependency_delta(
  baseline: &FxHashSet<PathBuf>,
  current: &FxHashSet<PathBuf>,
) -> DependencyDelta {
  DependencyDelta {
    added: current.difference(baseline).cloned().collect(),
    removed: baseline.difference(current).cloned().collect(),
  }
}

fn replay_dependency_delta(dependencies: &mut FxHashSet<PathBuf>, delta: &DependencyDelta) {
  dependencies.retain(|dependency| !delta.removed.contains(dependency));
  dependencies.extend(delta.added.iter().cloned());
}

fn record_pitch_data(
  loader_context: &mut LoaderContext<RunnerContext>,
  key: LoaderCacheKey,
  digest: String,
  resource: ResourceStamp,
) {
  let data = PitchData {
    key,
    digest,
    resource,
    diagnostics_len: loader_context.diagnostics.len(),
    file_dependencies: loader_context.file_dependencies.clone(),
    context_dependencies: loader_context.context_dependencies.clone(),
    missing_dependencies: loader_context.missing_dependencies.clone(),
    build_dependencies: loader_context.build_dependencies.clone(),
  };
  let index = loader_context.loader_index as usize;
  loader_context.loader_items[index]
    .set_data(serde_json::to_value(data).expect("cache loader pitch data should be serializable"));
}

#[cacheable]
#[derive(Debug, Default)]
struct CacheLoader;

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for CacheLoader {
  fn identifier(&self) -> rspack_collections::Identifier {
    INTERNAL_CACHE_LOADER_IDENTIFIER.into()
  }

  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(resource) = resource_stamp(loader_context).await else {
      return Ok(());
    };
    let (key, digest) = cache_key(loader_context);
    let entry = loader_context
      .context
      .loader_cache
      .lookup(&key, &digest, resource, loader_context.context.fs.as_ref())
      .await;
    if let Some(entry) = entry {
      let source_map = match entry.source_map {
        Some(source_map) => match SourceMap::from_json(source_map) {
          Ok(source_map) => Some(source_map),
          Err(_) => {
            loader_context
              .context
              .loader_cache
              .remove(&key, &digest)
              .await;
            record_pitch_data(loader_context, key, digest, resource);
            return Ok(());
          }
        },
        None => None,
      };
      replay_dependency_delta(
        &mut loader_context.file_dependencies,
        &entry.file_dependencies,
      );
      replay_dependency_delta(
        &mut loader_context.context_dependencies,
        &entry.context_dependencies,
      );
      replay_dependency_delta(
        &mut loader_context.missing_dependencies,
        &entry.missing_dependencies,
      );
      replay_dependency_delta(
        &mut loader_context.build_dependencies,
        &entry.build_dependencies,
      );
      loader_context.finish_with((entry.content, source_map, entry.additional_data));
      return Ok(());
    }

    record_pitch_data(loader_context, key, digest, resource);
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let pitch_data =
      serde_json::from_value::<PitchData>(loader_context.current_loader().data().clone()).ok();
    if let Some(pitch_data) = pitch_data
      && loader_context.cacheable
      && loader_context.diagnostics.len() == pitch_data.diagnostics_len
      && resource_stamp(loader_context).await == Some(pitch_data.resource)
    {
      let file_dependencies = dependency_delta(
        &pitch_data.file_dependencies,
        &loader_context.file_dependencies,
      );
      let context_dependencies = dependency_delta(
        &pitch_data.context_dependencies,
        &loader_context.context_dependencies,
      );
      let missing_dependencies = dependency_delta(
        &pitch_data.missing_dependencies,
        &loader_context.missing_dependencies,
      );
      let build_dependencies = dependency_delta(
        &pitch_data.build_dependencies,
        &loader_context.build_dependencies,
      );
      let mut loader_files = build_dependencies.added.clone();
      loader_files.extend(
        loader_context
          .loader_items
          .iter()
          .filter(|item| item.path().is_absolute())
          .map(|item| item.path().to_path_buf().into_std_path_buf()),
      );
      let build_dependencies = DependencyDelta {
        added: loader_files,
        removed: build_dependencies.removed,
      };
      if let Some(dependency_snapshot) = dependency_snapshots(
        &file_dependencies.added,
        &context_dependencies.added,
        &missing_dependencies.added,
        &build_dependencies.added,
        loader_context.context.fs.as_ref(),
      )
      .await
        && let Some(written_at_ns) = now_ns()
      {
        loader_context
          .context
          .loader_cache
          .store(
            pitch_data.key,
            pitch_data.digest,
            LoaderCacheEntry {
              resource: pitch_data.resource,
              written_at_ns,
              content: loader_context.content().cloned(),
              source_map: loader_context.source_map().map(SourceMap::to_json),
              additional_data: loader_context.additional_data().cloned(),
              file_dependencies,
              context_dependencies,
              missing_dependencies,
              build_dependencies,
              dependency_snapshot,
            },
            loader_context.context.fs.as_ref(),
          )
          .await;
      }
    }

    loader_context.current_loader().set_finish_called();
    Ok(())
  }
}

#[plugin]
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct LoaderCachePlugin;

impl LoaderCachePlugin {
  #[doc(hidden)]
  pub fn new() -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(NormalModuleFactoryCreateLoaderCache for LoaderCachePlugin)]
async fn create_loader_cache(&self, _loader: &ModuleRuleUseLoader) -> Result<Option<BoxLoader>> {
  Ok(Some(Arc::new(CacheLoader)))
}

impl Plugin for LoaderCachePlugin {
  fn name(&self) -> &'static str {
    "rspack.LoaderCachePlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .create_loader_cache
      .tap(create_loader_cache::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
  };

  use rspack_fs::MemoryFileSystem;
  use rspack_loader_runner::Content;
  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashSet;

  use super::{
    DependencyDelta, LoaderCacheEntry, LoaderCacheFileStore, LoaderCacheKey, LoaderCacheService,
    ResourceStamp, decode_entry, dependency_delta, encode_entry, entry_is_valid, now_ns,
    replay_dependency_delta,
  };

  static TEST_ID: AtomicU64 = AtomicU64::new(0);

  fn dependencies(values: &[&str]) -> FxHashSet<PathBuf> {
    values.iter().map(PathBuf::from).collect()
  }

  fn test_dir(name: &str) -> Utf8PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
    Utf8PathBuf::from_path_buf(std::env::temp_dir().join(format!(
      "rspack-loader-cache-{name}-{}-{id}",
      std::process::id()
    )))
    .expect("temp directory should be valid utf-8")
  }

  fn key() -> LoaderCacheKey {
    LoaderCacheKey {
      rspack_version: "test".to_string(),
      module_identifier: "module".to_string(),
      remaining_request: "loader!resource".to_string(),
    }
  }

  fn entry() -> LoaderCacheEntry {
    LoaderCacheEntry {
      resource: ResourceStamp {
        mtime_ns: 1,
        size: 3,
      },
      written_at_ns: now_ns().unwrap(),
      content: Some(Content::Buffer(vec![1, 2, 3])),
      source_map: None,
      additional_data: None,
      file_dependencies: DependencyDelta::default(),
      context_dependencies: DependencyDelta::default(),
      missing_dependencies: DependencyDelta::default(),
      build_dependencies: DependencyDelta::default(),
      dependency_snapshot: vec![],
    }
  }

  #[test]
  fn replays_dependency_additions_and_removals() {
    let baseline = dependencies(&["resource.js", "removed.js"]);
    let current = dependencies(&["resource.js", "added.js"]);
    let delta = dependency_delta(&baseline, &current);

    let mut replayed = baseline;
    replay_dependency_delta(&mut replayed, &delta);

    assert_eq!(replayed, current);
  }

  #[test]
  fn json_entry_round_trips_and_rejects_bad_data() {
    let key = key();
    let bytes = encode_entry(key.clone(), entry()).unwrap();
    let decoded = decode_entry(&bytes, &key).unwrap();
    assert_eq!(decoded.content, Some(Content::Buffer(vec![1, 2, 3])));

    assert!(decode_entry(b"{", &key).is_err());
    let mut envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    envelope["checksum"] = serde_json::json!(0);
    assert!(decode_entry(&serde_json::to_vec(&envelope).unwrap(), &key).is_err());
  }

  #[tokio::test]
  async fn file_store_uses_flat_paths_and_round_trips() {
    let root = test_dir("roundtrip");
    let store = LoaderCacheFileStore::new(root.clone(), false);
    store.put("abcdef", b"cached".to_vec()).await;

    assert_eq!(store.get("abcdef").await, Some(b"cached".to_vec()));
    assert!(root.join("abcdef.json").exists());
    assert!(!root.join("ab").exists());

    let _ = std::fs::remove_dir_all(root);
  }

  #[tokio::test]
  async fn concurrent_writers_leave_a_complete_entry() {
    let root = test_dir("concurrent");
    let store = LoaderCacheFileStore::new(root.clone(), false);
    let first = store.put("same", vec![1; 4096]);
    let second = store.put("same", vec![2; 4096]);
    tokio::join!(first, second);

    let bytes = store.get("same").await.unwrap();
    assert!(bytes == vec![1; 4096] || bytes == vec![2; 4096]);
    assert!(std::fs::read_dir(&root).unwrap().all(|entry| {
      !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains(".tmp.")
    }));

    let _ = std::fs::remove_dir_all(root);
  }

  #[tokio::test]
  async fn service_round_trips_between_instances_and_checks_mtime() {
    let root = test_dir("service-roundtrip");
    let store = LoaderCacheFileStore::new(root.clone(), false);
    let key = key();
    let digest = "digest".to_string();
    let entry = entry();
    LoaderCacheService::new(Some(store.clone()))
      .store(
        key.clone(),
        digest.clone(),
        entry.clone(),
        &MemoryFileSystem::default(),
      )
      .await;

    let reader = LoaderCacheService::new(Some(store));
    assert!(
      reader
        .lookup(&key, &digest, entry.resource, &MemoryFileSystem::default())
        .await
        .is_some()
    );
    assert!(
      reader
        .lookup(
          &key,
          &digest,
          ResourceStamp {
            mtime_ns: entry.resource.mtime_ns + 1,
            ..entry.resource
          },
          &MemoryFileSystem::default(),
        )
        .await
        .is_none()
    );

    let _ = std::fs::remove_dir_all(root);
  }

  #[tokio::test]
  async fn timestamp_rollback_is_invalid() {
    let mut entry = entry();
    let now = now_ns().unwrap();
    entry.written_at_ns = now + 10_000_000_000;
    assert!(!entry_is_valid(&entry, entry.resource, now, &MemoryFileSystem::default()).await);
  }

  #[tokio::test]
  async fn conditional_remove_does_not_delete_a_replaced_entry() {
    let root = test_dir("conditional-remove");
    let store = LoaderCacheFileStore::new(root.clone(), false);
    store.put("same", b"new".to_vec()).await;

    store.remove_if_unchanged("same", b"old".to_vec()).await;
    assert_eq!(store.get("same").await, Some(b"new".to_vec()));

    store.remove_if_unchanged("same", b"new".to_vec()).await;
    assert_eq!(store.get("same").await, None);

    let _ = std::fs::remove_dir_all(root);
  }
}

use std::{
  fs::{self, OpenOptions},
  io::{ErrorKind, Write},
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{AdditionalData, Content, Loader, LoaderContext, Scheme};
use rspack_paths::Utf8PathBuf;
use rspack_sources::SourceMap;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::{
  CacheOptions, CompilerOptions, RunnerContext, cache::persistent::storage::StorageOptions,
};

pub(crate) const INTERNAL_CACHE_LOADER_IDENTIFIER: &str = "builtin:cache-loader";

const FORMAT_VERSION: u8 = 1;
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_millis(500);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(10);
static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct LoaderCacheKey {
  rspack_version: String,
  compiler_scope: String,
  module_identifier: String,
  remaining_request: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct ResourceStamp {
  mtime_ms: u64,
  size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DependencyDelta {
  added: FxHashSet<PathBuf>,
  removed: FxHashSet<PathBuf>,
}

#[derive(Debug, Clone)]
struct LoaderCacheEntry {
  resource: ResourceStamp,
  content: Option<Content>,
  source_map: Option<String>,
  additional_data: Option<AdditionalData>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PersistedContent {
  String(String),
  Buffer(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedPayload {
  version: u8,
  identity: LoaderCacheKey,
  resource: ResourceStamp,
  content: PersistedContent,
  source_map: Option<String>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
}

#[derive(Debug, Serialize, Deserialize)]
struct PitchData {
  key: LoaderCacheKey,
  digest: String,
  resource: ResourceStamp,
  started_at_ms: u64,
  diagnostics_len: usize,
  file_dependencies: FxHashSet<PathBuf>,
  context_dependencies: FxHashSet<PathBuf>,
  missing_dependencies: FxHashSet<PathBuf>,
  build_dependencies: FxHashSet<PathBuf>,
}

#[derive(Debug, Clone)]
struct LoaderCacheFileStore {
  root: Utf8PathBuf,
  readonly: bool,
}

impl LoaderCacheFileStore {
  fn new(root: Utf8PathBuf, readonly: bool) -> Self {
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
  compiler_scope: String,
  entries: FxDashMap<LoaderCacheKey, LoaderCacheEntry>,
  file_store: Option<LoaderCacheFileStore>,
}

impl LoaderCacheService {
  pub(crate) fn from_compiler_options(
    compiler_path: &str,
    options: &CompilerOptions,
  ) -> Option<Self> {
    if !options.experiments.loader_cache {
      return None;
    }

    let (cache_version, file_store) = match &options.cache {
      CacheOptions::Persistent(option) => {
        let file_store = match &option.storage {
          StorageOptions::FileSystem { directory } => Some(LoaderCacheFileStore::new(
            directory.join("loader-cache/v1"),
            option.readonly,
          )),
        };
        (option.version.as_str(), file_store)
      }
      CacheOptions::Disabled | CacheOptions::Memory { .. } => ("", None),
    };
    let compiler_scope = format!(
      "{}\0{}\0{:?}\0{}\0{}",
      compiler_path,
      options.name.as_deref().unwrap_or_default(),
      options.mode,
      options.context,
      cache_version
    );

    Some(Self::new(compiler_scope, file_store))
  }

  fn new(compiler_scope: String, file_store: Option<LoaderCacheFileStore>) -> Self {
    Self {
      compiler_scope,
      entries: FxDashMap::default(),
      file_store,
    }
  }

  async fn lookup(
    &self,
    key: &LoaderCacheKey,
    digest: &str,
    resource: ResourceStamp,
  ) -> Option<LoaderCacheEntry> {
    if let Some(entry) = self.entries.get(key).map(|entry| entry.value().clone())
      && entry.resource == resource
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
    if entry.resource != resource {
      file_store.remove_if_unchanged(digest, bytes).await;
      return None;
    }
    self.entries.insert(key.clone(), entry.clone());
    Some(entry)
  }

  async fn store(&self, key: LoaderCacheKey, digest: String, entry: LoaderCacheEntry) {
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
    version: FORMAT_VERSION,
    identity: key,
    resource: entry.resource,
    content,
    source_map: entry.source_map,
    file_dependencies: entry.file_dependencies,
    context_dependencies: entry.context_dependencies,
    missing_dependencies: entry.missing_dependencies,
    build_dependencies: entry.build_dependencies,
  };
  serde_json::to_vec(&payload).ok()
}

fn decode_stored_entry(
  bytes: &[u8],
) -> std::result::Result<(LoaderCacheKey, LoaderCacheEntry), ()> {
  let payload: PersistedPayload = serde_json::from_slice(bytes).map_err(|_| ())?;
  if payload.version != FORMAT_VERSION {
    return Err(());
  }
  let key = payload.identity;
  let content = match payload.content {
    PersistedContent::String(value) => Content::String(value),
    PersistedContent::Buffer(value) => Content::Buffer(value),
  };
  Ok((
    key,
    LoaderCacheEntry {
      resource: payload.resource,
      content: Some(content),
      source_map: payload.source_map,
      additional_data: None,
      file_dependencies: payload.file_dependencies,
      context_dependencies: payload.context_dependencies,
      missing_dependencies: payload.missing_dependencies,
      build_dependencies: payload.build_dependencies,
    },
  ))
}

fn decode_entry(
  bytes: &[u8],
  expected_key: &LoaderCacheKey,
) -> std::result::Result<LoaderCacheEntry, ()> {
  let (key, entry) = decode_stored_entry(bytes)?;
  if &key != expected_key {
    return Err(());
  }
  Ok(entry)
}

fn now_ms() -> Option<u64> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .ok()?
    .as_millis()
    .try_into()
    .ok()
}

fn hash_bytes(value: &[u8]) -> u64 {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  hasher.write(value);
  hasher.finish()
}

fn cache_key(loader_context: &LoaderContext<RunnerContext>) -> (LoaderCacheKey, String) {
  let loader_cache = loader_context
    .context
    .loader_cache
    .as_ref()
    .expect("cache loader should only run when loader cache is enabled");
  let key = LoaderCacheKey {
    rspack_version: env!("CARGO_PKG_VERSION").to_string(),
    compiler_scope: loader_cache.compiler_scope.clone(),
    module_identifier: loader_context
      .context
      .module
      .identifier()
      .as_str()
      .to_owned(),
    remaining_request: loader_context.remaining_request().to_string(),
  };
  let bytes = serde_json::to_vec(&key).expect("loader cache key should be serializable");
  let digest = format!("{:016x}", hash_bytes(&bytes));
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
  Some(ResourceStamp {
    mtime_ms: metadata.mtime_ms,
    size: metadata.size,
  })
}

fn mtime_is_reliable(mtime_ms: u64, started_at_ms: u64) -> bool {
  // Match cache-loader's conservative handling for coarse filesystems: when
  // the resource mtime is in the same second as this cache attempt (or later),
  // it may have changed without producing a distinguishable timestamp.
  mtime_ms / 1000 < started_at_ms / 1000
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
    started_at_ms: now_ms().expect("system time should be after unix epoch"),
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
pub(crate) struct CacheLoader;

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
      .as_ref()
      .expect("cache loader should only run when loader cache is enabled")
      .lookup(&key, &digest, resource)
      .await;
    if let Some(entry) = entry {
      let source_map = match entry.source_map {
        Some(source_map) => match SourceMap::from_json(source_map) {
          Ok(source_map) => Some(source_map),
          Err(_) => {
            loader_context
              .context
              .loader_cache
              .as_ref()
              .expect("cache loader should only run when loader cache is enabled")
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
      && mtime_is_reliable(pitch_data.resource.mtime_ms, pitch_data.started_at_ms)
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
      loader_context
        .context
        .loader_cache
        .as_ref()
        .expect("cache loader should only run when loader cache is enabled")
        .store(
          pitch_data.key,
          pitch_data.digest,
          LoaderCacheEntry {
            resource: pitch_data.resource,
            content: loader_context.content().cloned(),
            source_map: loader_context.source_map().map(SourceMap::to_json),
            additional_data: loader_context.additional_data().cloned(),
            file_dependencies,
            context_dependencies,
            missing_dependencies,
            build_dependencies,
          },
        )
        .await;
    }

    loader_context.current_loader().set_finish_called();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
  };

  use rspack_loader_runner::Content;
  use rspack_paths::Utf8PathBuf;
  use rustc_hash::FxHashSet;

  use super::{
    DependencyDelta, LoaderCacheEntry, LoaderCacheFileStore, LoaderCacheKey, ResourceStamp,
    decode_stored_entry, dependency_delta, encode_entry, mtime_is_reliable,
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
      compiler_scope: "scope".to_string(),
      module_identifier: "module".to_string(),
      remaining_request: "loader!resource".to_string(),
    }
  }

  fn entry() -> LoaderCacheEntry {
    LoaderCacheEntry {
      resource: ResourceStamp {
        mtime_ms: 1,
        size: 3,
      },
      content: Some(Content::Buffer(vec![1, 2, 3])),
      source_map: None,
      additional_data: None,
      file_dependencies: DependencyDelta::default(),
      context_dependencies: DependencyDelta::default(),
      missing_dependencies: DependencyDelta::default(),
      build_dependencies: DependencyDelta::default(),
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
    let (decoded_key, decoded) = decode_stored_entry(&bytes).unwrap();
    assert_eq!(decoded_key, key);
    assert_eq!(decoded.content, Some(Content::Buffer(vec![1, 2, 3])));

    assert!(decode_stored_entry(b"{").is_err());
    let mut payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    payload["version"] = serde_json::json!(0);
    assert!(decode_stored_entry(&serde_json::to_vec(&payload).unwrap()).is_err());
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

  #[test]
  fn rejects_mtime_from_the_cache_attempt_second_or_later() {
    assert!(mtime_is_reliable(9_999, 10_000));
    assert!(!mtime_is_reliable(10_000, 10_999));
    assert!(!mtime_is_reliable(11_000, 10_999));
  }
}

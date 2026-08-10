use std::{
  hash::Hash,
  path::PathBuf,
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  time::{SystemTime, UNIX_EPOCH},
};

use rspack_fs::IntermediateFileSystem;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{
  AdditionalData, Content, LoaderChain, LoaderChainCacheAction, LoaderChainCacheState,
  LoaderContext,
};
use rspack_sources::SourceMap;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::{
  CacheOptions, CompilationAsset, CompilerOptions, Module, RunnerContext,
  cache::persistent::storage::StorageOptions,
};

const LOADER_CACHE_FORMAT_VERSION: u8 = 1;
static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct FileStamp {
  path: PathBuf,
  state: FileStampState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum FileStampState {
  Missing,
  Existing {
    mtime_ms: u64,
    ctime_ms: u64,
    size: u64,
    is_directory: bool,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct LoaderCacheKey {
  compiler_scope: String,
  static_fingerprint: String,
  input_hash: u64,
  loader_files: Vec<FileStamp>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DependencyDelta {
  added: FxHashSet<PathBuf>,
  removed: FxHashSet<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct JsonObjectDelta {
  upserted: serde_json::Map<String, serde_json::Value>,
  removed: FxHashSet<String>,
}

#[derive(Debug, Clone)]
struct LoaderCacheEntry {
  content: Option<Content>,
  source_map: Option<String>,
  additional_data: Option<AdditionalData>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  dependency_stamps: Vec<FileStamp>,
  parse_meta: rspack_loader_runner::ParseMeta,
  assets: FxHashMap<String, CompilationAsset>,
  build_info_extras: JsonObjectDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PersistedContent {
  String(String),
  Buffer(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedLoaderCacheEntry {
  key: LoaderCacheKey,
  content: PersistedContent,
  source_map: Option<String>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  dependency_stamps: Vec<FileStamp>,
  build_info_extras: JsonObjectDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedLoaderCacheEnvelope {
  format_version: u8,
  written_at_ms: u64,
  checksum: u64,
  entry: PersistedLoaderCacheEntry,
}

impl PersistedLoaderCacheEntry {
  fn from_memory(key: LoaderCacheKey, entry: &LoaderCacheEntry) -> Option<Self> {
    if entry.additional_data.is_some() || !entry.parse_meta.is_empty() || !entry.assets.is_empty() {
      return None;
    }
    let content = match entry.content.as_ref()? {
      Content::String(content) => PersistedContent::String(content.clone()),
      Content::Buffer(content) => PersistedContent::Buffer(content.clone()),
    };
    Some(Self {
      key,
      content,
      source_map: entry.source_map.clone(),
      file_dependencies: entry.file_dependencies.clone(),
      context_dependencies: entry.context_dependencies.clone(),
      missing_dependencies: entry.missing_dependencies.clone(),
      build_dependencies: entry.build_dependencies.clone(),
      dependency_stamps: entry.dependency_stamps.clone(),
      build_info_extras: entry.build_info_extras.clone(),
    })
  }

  fn into_memory(self, expected_key: &LoaderCacheKey) -> Option<LoaderCacheEntry> {
    if &self.key != expected_key {
      return None;
    }
    let content = match self.content {
      PersistedContent::String(content) => Content::String(content),
      PersistedContent::Buffer(content) => Content::Buffer(content),
    };
    Some(LoaderCacheEntry {
      content: Some(content),
      source_map: self.source_map,
      additional_data: None,
      file_dependencies: self.file_dependencies,
      context_dependencies: self.context_dependencies,
      missing_dependencies: self.missing_dependencies,
      build_dependencies: self.build_dependencies,
      dependency_stamps: self.dependency_stamps,
      parse_meta: Default::default(),
      assets: Default::default(),
      build_info_extras: self.build_info_extras,
    })
  }
}

#[derive(Debug, Clone)]
struct LoaderCacheFileStore {
  root: rspack_paths::Utf8PathBuf,
  readonly: bool,
  max_age: u64,
  fs: Arc<dyn IntermediateFileSystem>,
}

impl LoaderCacheFileStore {
  fn now_ms() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis() as u64
  }

  fn checksum(bytes: &[u8]) -> u64 {
    let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
    hasher.write(bytes);
    hasher.finish()
  }

  fn digest(key: &LoaderCacheKey) -> String {
    let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
    key.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
  }

  fn path(&self, key: &LoaderCacheKey) -> rspack_paths::Utf8PathBuf {
    self.root.join(format!("{}.json", Self::digest(key)))
  }

  async fn get(&self, key: &LoaderCacheKey) -> Option<LoaderCacheEntry> {
    let path = self.path(key);
    let bytes = self.fs.read_file(&path).await.ok()?;
    let envelope = serde_json::from_slice::<PersistedLoaderCacheEnvelope>(&bytes).ok()?;
    if envelope.format_version != LOADER_CACHE_FORMAT_VERSION {
      return None;
    }
    if Self::now_ms().saturating_sub(envelope.written_at_ms) > self.max_age.saturating_mul(1000) {
      if !self.readonly {
        let _ = self.fs.remove_file(&path).await;
      }
      return None;
    }
    let entry_bytes = serde_json::to_vec(&envelope.entry).ok()?;
    if Self::checksum(&entry_bytes) != envelope.checksum {
      return None;
    }
    envelope.entry.into_memory(key)
  }

  async fn put(&self, key: LoaderCacheKey, entry: &LoaderCacheEntry) {
    if self.readonly {
      return;
    }
    let Some(entry) = PersistedLoaderCacheEntry::from_memory(key.clone(), entry) else {
      // Never leave an older persistable result behind when the latest result
      // is memory-only.
      let _ = self.fs.remove_file(&self.path(&key)).await;
      return;
    };
    let Ok(entry_bytes) = serde_json::to_vec(&entry) else {
      return;
    };
    let envelope = PersistedLoaderCacheEnvelope {
      format_version: LOADER_CACHE_FORMAT_VERSION,
      written_at_ms: Self::now_ms(),
      checksum: Self::checksum(&entry_bytes),
      entry,
    };
    let Ok(bytes) = serde_json::to_vec(&envelope) else {
      return;
    };
    let path = self.path(&key);
    let Some(parent) = path.parent() else {
      return;
    };
    if self.fs.create_dir_all(parent).await.is_err() {
      return;
    }
    let temp_id = TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_extension(format!("json.tmp.{}.{}", std::process::id(), temp_id));
    if self.fs.write(&temp, &bytes).await.is_ok() && self.fs.rename(&temp, &path).await.is_err() {
      let _ = self.fs.remove_file(&path).await;
      let _ = self.fs.rename(&temp, &path).await;
    }
    let _ = self.fs.remove_file(&temp).await;
  }
}

pub(crate) struct LoaderCacheMissState {
  key: LoaderCacheKey,
  diagnostics_len: usize,
  file_dependencies: FxHashSet<PathBuf>,
  context_dependencies: FxHashSet<PathBuf>,
  missing_dependencies: FxHashSet<PathBuf>,
  build_dependencies: FxHashSet<PathBuf>,
  parse_meta_keys: FxHashSet<String>,
  asset_keys: FxHashSet<String>,
  build_info_extras: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug)]
pub(crate) struct LoaderCacheService {
  enabled: bool,
  compiler_scope: String,
  entries: FxDashMap<LoaderCacheKey, LoaderCacheEntry>,
  file_store: Option<LoaderCacheFileStore>,
}

impl LoaderCacheService {
  pub(crate) fn new(
    compiler_path: &str,
    options: &CompilerOptions,
    intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
  ) -> Self {
    let (enabled, version, file_store) = match &options.cache {
      CacheOptions::Disabled => (false, "", None),
      CacheOptions::Memory { .. } => (true, "", None),
      CacheOptions::Persistent(options) => {
        let file_store = match &options.storage {
          StorageOptions::FileSystem { directory } => Some(LoaderCacheFileStore {
            root: directory.join("loader-chain-cache/v1"),
            readonly: options.readonly,
            max_age: options.max_age,
            fs: intermediate_filesystem,
          }),
        };
        (true, options.version.as_str(), file_store)
      }
    };
    Self {
      enabled,
      compiler_scope: format!(
        "{}\0{compiler_path}\0{}\0{:?}\0{}\0{version}",
        rspack_workspace::rspack_pkg_version!(),
        options.name.as_deref().unwrap_or_default(),
        options.mode,
        options.context,
      ),
      entries: Default::default(),
      file_store,
    }
  }
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

fn json_object_delta(
  baseline: &serde_json::Map<String, serde_json::Value>,
  current: &serde_json::Map<String, serde_json::Value>,
) -> JsonObjectDelta {
  JsonObjectDelta {
    upserted: current
      .iter()
      .filter(|(key, value)| baseline.get(*key) != Some(*value))
      .map(|(key, value)| (key.clone(), value.clone()))
      .collect(),
    removed: baseline
      .keys()
      .filter(|key| !current.contains_key(*key))
      .cloned()
      .collect(),
  }
}

fn replay_json_object_delta(
  object: &mut serde_json::Map<String, serde_json::Value>,
  delta: &JsonObjectDelta,
) {
  object.retain(|key, _| !delta.removed.contains(key));
  object.extend(delta.upserted.clone());
}

async fn file_stamp(context: &RunnerContext, path: PathBuf) -> FileStamp {
  let path_utf8 = rspack_paths::Utf8Path::from_path(&path);
  let state = match path_utf8 {
    Some(path) => match context.fs.metadata(path).await {
      Ok(metadata) => FileStampState::Existing {
        mtime_ms: metadata.mtime_ms,
        ctime_ms: metadata.ctime_ms,
        size: metadata.size,
        is_directory: metadata.is_directory,
      },
      Err(_) => FileStampState::Missing,
    },
    None => FileStampState::Missing,
  };
  FileStamp { path, state }
}

async fn file_stamps(
  context: &RunnerContext,
  paths: impl IntoIterator<Item = PathBuf>,
) -> Vec<FileStamp> {
  let mut paths = paths.into_iter().collect::<Vec<_>>();
  paths.sort();
  paths.dedup();
  let mut stamps = Vec::with_capacity(paths.len());
  for path in paths {
    stamps.push(file_stamp(context, path).await);
  }
  stamps
}

async fn context_dependency_stamps(
  context: &RunnerContext,
  roots: impl IntoIterator<Item = PathBuf>,
) -> Vec<FileStamp> {
  let mut pending = roots.into_iter().collect::<Vec<_>>();
  let mut visited = FxHashSet::default();
  let mut stamps = Vec::new();
  while let Some(path) = pending.pop() {
    if !visited.insert(path.clone()) {
      continue;
    }
    let stamp = file_stamp(context, path.clone()).await;
    if matches!(
      stamp.state,
      FileStampState::Existing {
        is_directory: true,
        ..
      }
    ) && let Some(path) = rspack_paths::Utf8Path::from_path(&path)
      && let Ok(entries) = context.fs.read_dir(path).await
    {
      pending.extend(
        entries
          .into_iter()
          .map(|entry| path.join(entry).into_std_path_buf()),
      );
    }
    stamps.push(stamp);
  }
  stamps.sort_by(|left, right| left.path.cmp(&right.path));
  stamps
}

async fn stamps_are_valid(context: &RunnerContext, stamps: &[FileStamp]) -> bool {
  for stored in stamps {
    if file_stamp(context, stored.path.clone()).await.state != stored.state {
      return false;
    }
  }
  true
}

fn input_hash(content: &Content) -> u64 {
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  hasher.write(if content.is_string() {
    b"string"
  } else {
    b"buffer"
  });
  hasher.write(content.as_bytes());
  hasher.finish()
}

impl LoaderCacheService {
  async fn cache_key(
    &self,
    context: &mut LoaderContext<RunnerContext>,
    chain: &LoaderChain,
  ) -> Option<LoaderCacheKey> {
    // Source maps and AdditionalData are also loader inputs. Until both have
    // a stable input fingerprint, bypass instead of returning a partial hit.
    if context.source_map().is_some() || context.additional_data().is_some() {
      tracing::trace!(
        target: "rspack::loader_cache",
        reason = "unsupported-input-metadata",
        chain_len = chain.len(),
        "loader chain cache bypass"
      );
      return None;
    }
    let content = context.content()?;
    tracing::trace!(
      target: "rspack::loader_cache",
      input_bytes = content.as_bytes().len(),
      chain_len = chain.len(),
      "loader chain cache key"
    );
    let input_hash = input_hash(content);
    let loader_paths = context.loader_items[chain.range()]
      .iter()
      .filter(|loader| loader.path().is_absolute())
      .map(|loader| loader.path().as_std_path().to_path_buf())
      .collect::<Vec<_>>();
    context
      .build_dependencies
      .extend(loader_paths.iter().cloned());
    let loader_files = file_stamps(&context.context, loader_paths).await;
    Some(LoaderCacheKey {
      compiler_scope: self.compiler_scope.clone(),
      static_fingerprint: chain
        .static_fingerprint()
        .expect("loader cache service only accepts CacheChain")
        .to_owned(),
      input_hash,
      loader_files,
    })
  }

  pub(crate) async fn before_normal_chain(
    &self,
    context: &mut LoaderContext<RunnerContext>,
    chain: &LoaderChain,
  ) -> LoaderChainCacheAction {
    if !self.enabled {
      tracing::trace!(
        target: "rspack::loader_cache",
        reason = "compiler-cache-disabled",
        chain_len = chain.len(),
        "loader chain cache bypass"
      );
      return LoaderChainCacheAction::Disabled;
    }
    let Some(key) = self.cache_key(context, chain).await else {
      return LoaderChainCacheAction::Disabled;
    };

    let entry = if let Some(entry) = self.entries.get(&key) {
      Some(entry.value().clone())
    } else if let Some(file_store) = &self.file_store {
      let entry = file_store.get(&key).await;
      if let Some(entry) = &entry {
        self.entries.insert(key.clone(), entry.clone());
      }
      entry
    } else {
      None
    };
    if let Some(entry) = entry {
      if !stamps_are_valid(&context.context, &entry.dependency_stamps).await {
        tracing::trace!(
          target: "rspack::loader_cache",
          reason = "dependency-changed",
          chain_len = chain.len(),
          "loader chain cache stale"
        );
        self.entries.remove(&key);
      } else {
        let source_map = match entry.source_map {
          Some(source_map) => match SourceMap::from_json(source_map) {
            Ok(source_map) => Some(source_map),
            Err(_) => {
              self.entries.remove(&key);
              return self.miss(context, key);
            }
          },
          None => None,
        };
        replay_dependency_delta(&mut context.file_dependencies, &entry.file_dependencies);
        replay_dependency_delta(
          &mut context.context_dependencies,
          &entry.context_dependencies,
        );
        replay_dependency_delta(
          &mut context.missing_dependencies,
          &entry.missing_dependencies,
        );
        replay_dependency_delta(&mut context.build_dependencies, &entry.build_dependencies);
        context.parse_meta.extend(entry.parse_meta);
        context
          .context
          .module
          .build_info_mut()
          .assets
          .extend(entry.assets);
        replay_json_object_delta(
          &mut context.context.module.build_info_mut().extras,
          &entry.build_info_extras,
        );
        context.__finish_with((entry.content, source_map, entry.additional_data));
        tracing::trace!(
          target: "rspack::loader_cache",
          chain_len = chain.len(),
          "loader chain cache hit"
        );
        return LoaderChainCacheAction::Hit;
      }
    }

    tracing::trace!(
      target: "rspack::loader_cache",
      chain_len = chain.len(),
      "loader chain cache miss"
    );
    self.miss(context, key)
  }

  fn miss(
    &self,
    context: &LoaderContext<RunnerContext>,
    key: LoaderCacheKey,
  ) -> LoaderChainCacheAction {
    LoaderChainCacheAction::Miss(LoaderChainCacheState::new(LoaderCacheMissState {
      key,
      diagnostics_len: context.diagnostics.len(),
      file_dependencies: context.file_dependencies.clone(),
      context_dependencies: context.context_dependencies.clone(),
      missing_dependencies: context.missing_dependencies.clone(),
      build_dependencies: context.build_dependencies.clone(),
      parse_meta_keys: context.parse_meta.keys().cloned().collect(),
      asset_keys: context
        .context
        .module
        .build_info()
        .assets
        .keys()
        .cloned()
        .collect(),
      build_info_extras: context.context.module.build_info().extras.clone(),
    }))
  }

  pub(crate) async fn after_normal_chain(
    &self,
    context: &mut LoaderContext<RunnerContext>,
    state: LoaderCacheMissState,
  ) {
    if !context.cacheable || context.diagnostics.len() != state.diagnostics_len {
      tracing::trace!(
        target: "rspack::loader_cache",
        reason = if !context.cacheable { "cacheable-false" } else { "diagnostics" },
        "loader chain cache bypass"
      );
      return;
    }

    let file_dependencies = dependency_delta(&state.file_dependencies, &context.file_dependencies);
    let context_dependencies =
      dependency_delta(&state.context_dependencies, &context.context_dependencies);
    let missing_dependencies =
      dependency_delta(&state.missing_dependencies, &context.missing_dependencies);
    let build_dependencies =
      dependency_delta(&state.build_dependencies, &context.build_dependencies);
    // Stamp the full post-chain dependency sets. A dependency may already have
    // been registered by a loader to the right, so stamping only set deltas
    // would miss a dependency that this chain also observes.
    let dependency_paths = context
      .file_dependencies
      .iter()
      .chain(&context.missing_dependencies)
      .chain(&context.build_dependencies)
      .cloned()
      .collect::<Vec<_>>();
    let mut dependency_stamps = file_stamps(&context.context, dependency_paths).await;
    dependency_stamps.extend(
      context_dependency_stamps(
        &context.context,
        context.context_dependencies.iter().cloned(),
      )
      .await,
    );
    dependency_stamps.sort_by(|left, right| left.path.cmp(&right.path));
    dependency_stamps.dedup_by(|left, right| left.path == right.path);

    // A dependency changing while the chain is running makes the candidate
    // ambiguous. Do not publish it.
    if !stamps_are_valid(&context.context, &dependency_stamps).await {
      tracing::trace!(
        target: "rspack::loader_cache",
        reason = "dependency-changed-during-execution",
        "loader chain cache bypass"
      );
      return;
    }

    let parse_meta = context
      .parse_meta
      .iter()
      .filter(|(key, _)| !state.parse_meta_keys.contains(*key))
      .map(|(key, value)| (key.clone(), value.clone()))
      .collect();
    let assets = context
      .context
      .module
      .build_info()
      .assets
      .iter()
      .filter(|(key, _)| !state.asset_keys.contains(*key))
      .map(|(key, value)| (key.clone(), value.clone()))
      .collect();
    let build_info_extras = json_object_delta(
      &state.build_info_extras,
      &context.context.module.build_info().extras,
    );

    let entry = LoaderCacheEntry {
      content: context.content().cloned(),
      source_map: context.source_map().map(SourceMap::to_json),
      additional_data: context.additional_data().cloned(),
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      build_dependencies,
      dependency_stamps,
      parse_meta,
      assets,
      build_info_extras,
    };
    self.entries.insert(state.key.clone(), entry.clone());
    tracing::trace!(target: "rspack::loader_cache", "loader chain cache store");
    if let Some(file_store) = &self.file_store {
      file_store.put(state.key, &entry).await;
    }
  }
}

pub(crate) type SharedLoaderCacheService = Arc<LoaderCacheService>;

use std::{
  path::PathBuf,
  sync::{Arc, LazyLock},
};

use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, AsPreset, AsVec},
};
use rspack_collections::Identifiable;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{
  Content, LoaderChain, LoaderChainCacheAction, LoaderChainCacheState, LoaderContext,
};
use rspack_paths::Utf8PathBuf;
use rspack_sources::SourceMap;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashSet;

use crate::{
  Compilation, Module, RunnerContext,
  new_cache::{CacheFacade, CacheValue, Etag},
};

const LOADER_CACHE_DIRECTORY: &str = "node_modules/.cache/lodaer-cache";

// Keep a strong process-wide owner so compilers using the same cache directory
// always share exactly one LoaderCache instance.
static LOADER_CACHES: LazyLock<FxDashMap<Utf8PathBuf, Arc<LoaderCache>>> =
  LazyLock::new(FxDashMap::default);

#[derive(Debug)]
pub struct LoaderCache {
  storage: CacheFacade,
}

impl LoaderCache {
  fn new(storage: CacheFacade) -> Self {
    Self { storage }
  }

  fn cache_identifier(chain_key: &str, module_identifier: &str) -> String {
    fn push_segment(key: &mut String, segment: &str) {
      key.push('|');
      key.push_str(&segment.len().to_string());
      key.push(':');
      key.push_str(segment);
    }

    let mut key = String::new();
    push_segment(&mut key, chain_key);
    push_segment(&mut key, module_identifier);
    key
  }

  fn get(
    &self,
    chain_key: &str,
    module_identifier: &str,
    etag: &Etag,
  ) -> Option<CacheValue<LoaderCacheEntry>> {
    let identifier = Self::cache_identifier(chain_key, module_identifier);
    match self.storage.get(&identifier, Some(etag.clone())) {
      Ok(value) => value,
      Err(error) => {
        tracing::warn!("Restoring loader cache entry failed: {error}");
        None
      }
    }
  }

  fn store(
    &self,
    chain_key: &str,
    module_identifier: &str,
    etag: Etag,
    value: CacheValue<LoaderCacheEntry>,
  ) {
    let identifier = Self::cache_identifier(chain_key, module_identifier);
    if let Err(error) = self.storage.store(&identifier, Some(etag), value) {
      tracing::warn!("Storing loader cache entry failed: {error}");
    }
  }
}

pub fn get_loader_cache(compilation: &Compilation) -> Arc<LoaderCache> {
  let cache_directory = compilation
    .options
    .context
    .as_path()
    .join(LOADER_CACHE_DIRECTORY);
  LOADER_CACHES
    .entry(cache_directory)
    .or_insert_with(|| Arc::new(LoaderCache::new(compilation.get_cache("LoaderCache"))))
    .clone()
}

#[cacheable]
#[derive(Clone, Default)]
struct DependencyDelta {
  #[cacheable(with=AsVec<As<PortablePath>>)]
  added: FxHashSet<PathBuf>,
  #[cacheable(with=AsVec<As<PortablePath>>)]
  removed: FxHashSet<PathBuf>,
}

#[cacheable]
#[derive(Clone, Default)]
struct JsonObjectDelta {
  #[cacheable(with=AsPreset)]
  upserted: serde_json::Map<String, serde_json::Value>,
  #[cacheable(with=AsVec)]
  removed: FxHashSet<String>,
}

#[cacheable]
#[derive(Clone)]
enum LoaderCacheContent {
  String(String),
  Buffer(Vec<u8>),
}

impl From<&Content> for LoaderCacheContent {
  fn from(value: &Content) -> Self {
    match value {
      Content::String(value) => Self::String(value.clone()),
      Content::Buffer(value) => Self::Buffer(value.clone()),
    }
  }
}

impl From<LoaderCacheContent> for Content {
  fn from(value: LoaderCacheContent) -> Self {
    match value {
      LoaderCacheContent::String(value) => Self::String(value),
      LoaderCacheContent::Buffer(value) => Self::Buffer(value),
    }
  }
}

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<LoaderCacheContent>,
  source_map: Option<String>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  build_info_extras: JsonObjectDelta,
}

pub(crate) struct LoaderCacheMissState {
  cache: Arc<LoaderCache>,
  cache_key: String,
  module_identifier: String,
  etag: Etag,
  diagnostics_len: usize,
  file_dependencies: FxHashSet<PathBuf>,
  context_dependencies: FxHashSet<PathBuf>,
  missing_dependencies: FxHashSet<PathBuf>,
  build_dependencies: FxHashSet<PathBuf>,
  build_info_extras: serde_json::Map<String, serde_json::Value>,
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

fn input_etag(context: &LoaderContext<RunnerContext>) -> Option<Etag> {
  if context.additional_data().is_some() {
    return None;
  }
  let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
  match context.content()? {
    Content::String(content) => {
      hasher.write(b"string");
      hasher.write(content.as_bytes());
    }
    Content::Buffer(content) => {
      hasher.write(b"buffer");
      hasher.write(content);
    }
  }
  if let Some(source_map) = context.source_map() {
    hasher.write(b"source-map");
    hasher.write(source_map.to_json().as_bytes());
  }
  Some(Etag::from(format!("{:016x}", hasher.finish())))
}

pub(crate) fn before_normal_chain(
  context: &mut LoaderContext<RunnerContext>,
  chain: &LoaderChain,
) -> LoaderChainCacheAction {
  let Some(etag) = input_etag(context) else {
    return LoaderChainCacheAction::Disabled;
  };
  // parseMeta cannot be compared generically and emitted assets are observable
  // side effects. A chain that starts after either value exists is not cached.
  if !context.parse_meta.is_empty() || !context.context.module.build_info().assets.is_empty() {
    return LoaderChainCacheAction::Disabled;
  }
  let Some(cache_key) = context.loader_chain_cache_key(chain) else {
    return LoaderChainCacheAction::Disabled;
  };
  let module_identifier = context.context.module.identifier();
  let cache = Arc::clone(&context.context.loader_cache);

  if let Some(entry) = cache.get(&cache_key, module_identifier.as_str(), &etag) {
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
    replay_json_object_delta(
      &mut context.context.module.build_info_mut().extras,
      &entry.build_info_extras,
    );
    let source_map = entry
      .source_map
      .clone()
      .and_then(|source_map| SourceMap::from_json(source_map).ok());
    context.__finish_with((entry.content.clone().map(Content::from), source_map, None));
    return LoaderChainCacheAction::Hit;
  }

  LoaderChainCacheAction::Miss(LoaderChainCacheState::new(LoaderCacheMissState {
    cache,
    cache_key,
    module_identifier: module_identifier.as_str().to_owned(),
    etag,
    diagnostics_len: context.diagnostics.len(),
    file_dependencies: context.file_dependencies.clone(),
    context_dependencies: context.context_dependencies.clone(),
    missing_dependencies: context.missing_dependencies.clone(),
    build_dependencies: context.build_dependencies.clone(),
    build_info_extras: context.context.module.build_info().extras.clone(),
  }))
}

pub(crate) fn after_normal_chain(
  context: &LoaderContext<RunnerContext>,
  state: LoaderCacheMissState,
) {
  // AdditionalData and ParseMeta are open, process-local type maps without a
  // stable serialization contract. Chains producing either value are left
  // uncached instead of leaking memory-only exceptions into the cache layer.
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
    || context.additional_data().is_some()
    || !context.parse_meta.is_empty()
  {
    return;
  }

  // V1 intentionally does not fingerprint loader dependencies. We replay the
  // dependency deltas so watch registration is preserved, but a dependency's
  // contents are not part of the cache key yet. Loaders whose output depends
  // on an added dependency are therefore outside the supported scope for now.
  let entry = LoaderCacheEntry {
    content: context.content().map(LoaderCacheContent::from),
    source_map: context.source_map().map(SourceMap::to_json),
    file_dependencies: dependency_delta(&state.file_dependencies, &context.file_dependencies),
    context_dependencies: dependency_delta(
      &state.context_dependencies,
      &context.context_dependencies,
    ),
    missing_dependencies: dependency_delta(
      &state.missing_dependencies,
      &context.missing_dependencies,
    ),
    build_dependencies: dependency_delta(&state.build_dependencies, &context.build_dependencies),
    build_info_extras: json_object_delta(
      &state.build_info_extras,
      &context.context.module.build_info().extras,
    ),
  };
  state.cache.store(
    &state.cache_key,
    &state.module_identifier,
    state.etag,
    CacheValue::new(entry),
  );
}

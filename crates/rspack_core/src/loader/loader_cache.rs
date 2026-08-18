use std::{
  any::Any,
  path::PathBuf,
  sync::{Arc, LazyLock},
};

use rspack_collections::Identifiable;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{
  AdditionalData, Content, LoaderCacheAction, LoaderCacheState, LoaderContext, ParseMeta,
};
use rspack_paths::Utf8PathBuf;
use rspack_sources::SourceMap;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashSet;

use crate::{
  BoxLoader, Context, Module, RunnerContext,
  new_cache::{CacheKey, CacheValue, Etag, MemoryCache, MemoryCacheGetResult},
};

const LOADER_CACHE_DIRECTORY: &str = "node_modules/.cache/loader-cache";

// Keep a strong process-wide owner so compilers using the same cache directory
// always share exactly one LoaderCache instance.
static LOADER_CACHES: LazyLock<FxDashMap<Utf8PathBuf, Arc<LoaderCache>>> =
  LazyLock::new(FxDashMap::default);

#[derive(Debug)]
pub struct LoaderCache {
  // V1 uses this directory as the cache instance identity only. The storage is
  // still memory-only; a later persistent version can remain behind this type.
  cache_directory: Utf8PathBuf,
  storage: MemoryCache,
}

impl LoaderCache {
  fn new(cache_directory: Utf8PathBuf) -> Self {
    Self {
      cache_directory,
      storage: MemoryCache::default(),
    }
  }

  fn cache_key(&self, loader_key: &str, module_identifier: &str) -> CacheKey {
    fn push_segment(key: &mut String, segment: &str) {
      key.push('|');
      key.push_str(&segment.len().to_string());
      key.push(':');
      key.push_str(segment);
    }

    let mut key = self.cache_directory.as_str().to_owned();
    push_segment(&mut key, loader_key);
    push_segment(&mut key, module_identifier);
    CacheKey::from(key)
  }

  #[doc(hidden)]
  pub fn get<T: Any + Send + Sync>(
    &self,
    loader_key: &str,
    module_identifier: &str,
    etag: &Etag,
  ) -> Option<CacheValue<T>> {
    let key = self.cache_key(loader_key, module_identifier);
    match self.storage.get(&key, Some(etag)) {
      MemoryCacheGetResult::Hit(value) => Some(value),
      MemoryCacheGetResult::Miss | MemoryCacheGetResult::NotCached => None,
    }
  }

  #[doc(hidden)]
  pub fn store<T: Any + Send + Sync>(
    &self,
    loader_key: &str,
    module_identifier: &str,
    etag: Etag,
    value: CacheValue<T>,
  ) {
    let key = self.cache_key(loader_key, module_identifier);
    self.storage.store(key, Some(etag), value);
  }
}

pub fn get_loader_cache(context: &Context) -> Arc<LoaderCache> {
  let cache_directory = context.as_path().join(LOADER_CACHE_DIRECTORY);
  LOADER_CACHES
    .entry(cache_directory.clone())
    .or_insert_with(|| Arc::new(LoaderCache::new(cache_directory)))
    .clone()
}

pub(crate) fn loader_cache_key(name: &str, loader: &BoxLoader, options: &str) -> String {
  let version = loader
    .cache_version()
    .unwrap_or(rspack_workspace::rspack_pkg_version!());
  format!(
    "{}:{name}{}:{options}{}:{version}",
    name.len(),
    options.len(),
    version.len()
  )
}

#[derive(Clone, Default)]
struct DependencyDelta {
  added: FxHashSet<PathBuf>,
  removed: FxHashSet<PathBuf>,
}

#[derive(Clone, Default)]
struct JsonObjectDelta {
  upserted: serde_json::Map<String, serde_json::Value>,
  removed: FxHashSet<String>,
}

#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Content>,
  source_map: Option<String>,
  additional_data: Option<AdditionalData>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  parse_meta: ParseMeta,
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

pub(crate) fn before_normal_loader(
  context: &mut LoaderContext<RunnerContext>,
) -> LoaderCacheAction {
  if !context.cacheable {
    return LoaderCacheAction::Disabled;
  }
  let Some(etag) = input_etag(context) else {
    return LoaderCacheAction::Disabled;
  };
  // parseMeta cannot be compared generically and emitted assets are observable
  // side effects. A loader that starts after either value exists is not cached.
  if !context.parse_meta.is_empty() || !context.context.module.build_info().assets.is_empty() {
    return LoaderCacheAction::Disabled;
  }
  let cache_key = context.current_loader().cache_key().to_owned();
  let module_identifier = context.context.module.identifier();
  let cache = Arc::clone(&context.context.loader_cache);

  if let Some(entry) = cache.get::<LoaderCacheEntry>(&cache_key, module_identifier.as_str(), &etag)
  {
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
    context.parse_meta.extend(entry.parse_meta.clone());
    replay_json_object_delta(
      &mut context.context.module.build_info_mut().extras,
      &entry.build_info_extras,
    );
    let source_map = entry
      .source_map
      .clone()
      .and_then(|source_map| SourceMap::from_json(source_map).ok());
    context.__finish_with((
      entry.content.clone(),
      source_map,
      entry.additional_data.clone(),
    ));
    return LoaderCacheAction::Hit;
  }

  LoaderCacheAction::Miss(LoaderCacheState::new(LoaderCacheMissState {
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

pub(crate) fn after_normal_loader(
  context: &LoaderContext<RunnerContext>,
  state: LoaderCacheMissState,
) {
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
  {
    return;
  }

  // V1 intentionally does not fingerprint loader dependencies. We replay the
  // dependency deltas so watch registration is preserved, but a dependency's
  // contents are not part of the cache key yet. Loaders whose output depends
  // on an added dependency are therefore outside the supported scope for now.
  let entry = LoaderCacheEntry {
    content: context.content().cloned(),
    source_map: context.source_map().map(SourceMap::to_json),
    additional_data: context.additional_data().cloned(),
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
    parse_meta: context.parse_meta.clone(),
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

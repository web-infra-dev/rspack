use std::{
  any::Any,
  fmt,
  path::PathBuf,
  sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, AsPreset, AsVec},
};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_loader_runner::{
  AdditionalData, Content, LoaderCacheAction, LoaderCacheState, LoaderContext, ParseMeta,
};
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet;
use ustr::Ustr;

use crate::{
  BoxLoader, CacheFacade, CacheValue, Context, Etag, ItemCacheFacade, Module, RunnerContext,
};

const LOADER_CACHE_DIRECTORY: &str = "node_modules/.cache/loader-cache";

// Keep a strong process-wide owner so compilers using the same cache directory
// always share exactly one LoaderCache instance.
static LOADER_CACHES: LazyLock<DashMap<Ustr, Arc<LoaderCache>>> = LazyLock::new(DashMap::new);

#[derive(Debug)]
pub struct LoaderCache {
  cache_directory: Ustr,
  storage: CacheFacade,
  sidecars: DashMap<String, LoaderCacheSidecarEntry>,
}

impl LoaderCache {
  fn new(cache_directory: Ustr, storage: CacheFacade) -> Self {
    Self {
      cache_directory,
      storage,
      sidecars: DashMap::new(),
    }
  }

  fn cache_key(&self, loader_key: &str, module_identifier: &str) -> String {
    fn push_segment(key: &mut String, segment: &str) {
      key.push('|');
      key.push_str(&segment.len().to_string());
      key.push(':');
      key.push_str(segment);
    }

    let mut key = self.cache_directory.as_str().to_owned();
    push_segment(&mut key, loader_key);
    push_segment(&mut key, module_identifier);
    key
  }

  #[doc(hidden)]
  pub fn cache_item(
    &self,
    loader_key: &str,
    module_identifier: &str,
    etag: Etag,
  ) -> (String, ItemCacheFacade) {
    let key = self.cache_key(loader_key, module_identifier);
    let item = self.storage.get_item_cache(&key, Some(etag));
    (key, item)
  }

  #[doc(hidden)]
  pub fn get_sidecar<T: Any + Send + Sync>(&self, key: &str, etag: &Etag) -> Option<Arc<T>> {
    let entry = self.sidecars.get(key)?;
    if &entry.etag != etag {
      return None;
    }
    Arc::clone(&entry.value).downcast().ok()
  }

  #[doc(hidden)]
  pub fn store_sidecar<T: Any + Send + Sync>(&self, key: String, etag: Etag, value: T) {
    self.sidecars.insert(
      key,
      LoaderCacheSidecarEntry {
        etag,
        value: Arc::new(value),
      },
    );
  }

  #[doc(hidden)]
  pub fn remove_sidecar(&self, key: &str) {
    self.sidecars.remove(key);
  }
}

struct LoaderCacheSidecarEntry {
  etag: Etag,
  value: Arc<dyn Any + Send + Sync>,
}

impl fmt::Debug for LoaderCacheSidecarEntry {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("LoaderCacheSidecarEntry")
      .field("etag", &self.etag)
      .finish_non_exhaustive()
  }
}

fn loader_cache_directory(context: &Context) -> Ustr {
  Ustr::from(context.as_path().join(LOADER_CACHE_DIRECTORY).as_str())
}

pub(crate) fn register_loader_cache(context: &Context, storage: CacheFacade) {
  let cache_directory = loader_cache_directory(context);
  LOADER_CACHES
    .entry(cache_directory)
    .or_insert_with(|| Arc::new(LoaderCache::new(cache_directory, storage)));
}

pub fn get_loader_cache(context: &Context) -> Arc<LoaderCache> {
  let cache_directory = loader_cache_directory(context);
  LOADER_CACHES
    .get(&cache_directory)
    .map(|cache| Arc::clone(cache.value()))
    .expect("loader cache should be registered when the compilation is created")
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

#[cacheable]
#[derive(Clone, Default)]
struct DependencyDelta {
  #[cacheable(with=AsVec<As<PortablePath>>)]
  added: Vec<PathBuf>,
  #[cacheable(with=AsVec<As<PortablePath>>)]
  removed: Vec<PathBuf>,
}

#[cacheable]
#[derive(Clone, Default)]
struct JsonObjectDelta {
  #[cacheable(with=AsPreset)]
  upserted: serde_json::Map<String, serde_json::Value>,
  removed: Vec<String>,
}

#[cacheable]
#[derive(Clone)]
struct LoaderCacheEntry {
  content: Option<Vec<u8>>,
  content_is_string: bool,
  source_map: Option<String>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
  build_info_extras: JsonObjectDelta,
  requires_sidecar: bool,
}

#[derive(Clone)]
struct LoaderCacheSidecar {
  additional_data: Option<AdditionalData>,
  parse_meta: ParseMeta,
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

fn cache_miss_action(
  context: &LoaderContext<RunnerContext>,
  cache: Arc<LoaderCache>,
  cache_key: String,
  module_identifier: &str,
  etag: Etag,
) -> LoaderCacheAction {
  LoaderCacheAction::Miss(LoaderCacheState::new(LoaderCacheMissState {
    cache,
    cache_key,
    module_identifier: module_identifier.to_owned(),
    etag,
    diagnostics_len: context.diagnostics.len(),
    file_dependencies: context.file_dependencies.clone(),
    context_dependencies: context.context_dependencies.clone(),
    missing_dependencies: context.missing_dependencies.clone(),
    build_dependencies: context.build_dependencies.clone(),
    build_info_extras: context.context.module.build_info().extras.clone(),
  }))
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
) -> Result<LoaderCacheAction> {
  if !context.cacheable {
    return Ok(LoaderCacheAction::Disabled);
  }
  let Some(etag) = input_etag(context) else {
    return Ok(LoaderCacheAction::Disabled);
  };
  // parseMeta cannot be compared generically and emitted assets are observable
  // side effects. A loader that starts after either value exists is not cached.
  if !context.parse_meta.is_empty() || !context.context.module.build_info().assets.is_empty() {
    return Ok(LoaderCacheAction::Disabled);
  }
  let cache_key = context.current_loader().cache_key().to_owned();
  let module_identifier = context.context.module.identifier();
  let cache = get_loader_cache(&context.context.options.context);
  let (storage_key, item_cache) =
    cache.cache_item(&cache_key, module_identifier.as_str(), etag.clone());

  if let Some(entry) = item_cache.get::<LoaderCacheEntry>()? {
    let sidecar = if entry.requires_sidecar {
      let Some(sidecar) = cache.get_sidecar::<LoaderCacheSidecar>(&storage_key, &etag) else {
        return Ok(cache_miss_action(
          context,
          cache,
          cache_key,
          module_identifier.as_str(),
          etag,
        ));
      };
      Some(sidecar)
    } else {
      None
    };
    let content = match (&entry.content, entry.content_is_string) {
      (Some(content), true) => {
        let Ok(content) = String::from_utf8(content.clone()) else {
          return Ok(cache_miss_action(
            context,
            cache,
            cache_key,
            module_identifier.as_str(),
            etag,
          ));
        };
        Some(Content::String(content))
      }
      (Some(content), false) => Some(Content::Buffer(content.clone())),
      (None, _) => None,
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
    if let Some(sidecar) = &sidecar {
      context.parse_meta.extend(sidecar.parse_meta.clone());
    }
    replay_json_object_delta(
      &mut context.context.module.build_info_mut().extras,
      &entry.build_info_extras,
    );
    let source_map = entry
      .source_map
      .clone()
      .and_then(|source_map| SourceMap::from_json(source_map).ok());
    context.__finish_with((
      content,
      source_map,
      sidecar.and_then(|sidecar| sidecar.additional_data.clone()),
    ));
    return Ok(LoaderCacheAction::Hit);
  }

  Ok(cache_miss_action(
    context,
    cache,
    cache_key,
    module_identifier.as_str(),
    etag,
  ))
}

pub(crate) fn after_normal_loader(
  context: &LoaderContext<RunnerContext>,
  state: LoaderCacheMissState,
) -> Result<()> {
  if !context.cacheable
    || context.diagnostics.len() != state.diagnostics_len
    || !context.context.module.build_info().assets.is_empty()
  {
    return Ok(());
  }

  // V1 intentionally does not fingerprint loader dependencies. We replay the
  // dependency deltas so watch registration is preserved, but a dependency's
  // contents are not part of the cache key yet. Loaders whose output depends
  // on an added dependency are therefore outside the supported scope for now.
  let (content, content_is_string) = match context.content() {
    Some(Content::String(content)) => (Some(content.as_bytes().to_vec()), true),
    Some(Content::Buffer(content)) => (Some(content.clone()), false),
    None => (None, false),
  };
  let sidecar = LoaderCacheSidecar {
    additional_data: context.additional_data().cloned(),
    parse_meta: context.parse_meta.clone(),
  };
  let requires_sidecar = sidecar.additional_data.is_some() || !sidecar.parse_meta.is_empty();
  let entry = LoaderCacheEntry {
    content,
    content_is_string,
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
    requires_sidecar,
  };
  let (storage_key, item_cache) = state.cache.cache_item(
    &state.cache_key,
    &state.module_identifier,
    state.etag.clone(),
  );
  if requires_sidecar {
    state.cache.store_sidecar(storage_key, state.etag, sidecar);
  } else {
    state.cache.remove_sidecar(&storage_key);
  }
  item_cache.store(CacheValue::new(entry))
}

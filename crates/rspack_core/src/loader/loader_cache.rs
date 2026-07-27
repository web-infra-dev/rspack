use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Unsupported};
use rspack_collections::Identifiable;
use rspack_error::Result;
use rspack_hash::{HashFunction, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::{AdditionalData, Content, Loader, LoaderContext, Scheme};
use rspack_sources::SourceMap;
use rspack_util::fx_hash::FxDashMap;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::{
  ApplyContext, BoxLoader, ModuleRuleUseLoader, NormalModuleFactoryCreateLoaderCache, Plugin,
  RunnerContext,
};

pub(crate) const INTERNAL_CACHE_LOADER_IDENTIFIER: &str = "builtin:cache-loader";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct LoaderCacheKey {
  module_identifier: String,
  remaining_request: String,
}

#[derive(Debug, Clone)]
struct DependencyDelta {
  added: FxHashSet<PathBuf>,
  removed: FxHashSet<PathBuf>,
}

#[derive(Debug, Clone)]
struct LoaderCacheEntry {
  resource_hash: u64,
  content: Option<Content>,
  source_map: Option<String>,
  additional_data: Option<AdditionalData>,
  file_dependencies: DependencyDelta,
  context_dependencies: DependencyDelta,
  missing_dependencies: DependencyDelta,
  build_dependencies: DependencyDelta,
}

#[derive(Debug, Default)]
struct LoaderCache {
  entries: FxDashMap<LoaderCacheKey, LoaderCacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PitchData {
  key: LoaderCacheKey,
  resource_hash: u64,
  diagnostics_len: usize,
  file_dependencies: FxHashSet<PathBuf>,
  context_dependencies: FxHashSet<PathBuf>,
  missing_dependencies: FxHashSet<PathBuf>,
  build_dependencies: FxHashSet<PathBuf>,
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

#[cacheable]
#[derive(Debug)]
struct CacheLoader {
  #[cacheable(with=Unsupported)]
  cache: Arc<LoaderCache>,
}

impl CacheLoader {
  fn new(cache: Arc<LoaderCache>) -> Self {
    Self { cache }
  }

  async fn resource_hash(loader_context: &LoaderContext<RunnerContext>) -> Option<u64> {
    if loader_context.resource_data().get_scheme() != &Scheme::None {
      return None;
    }
    let resource_path = loader_context.resource_path()?.to_path_buf();
    if resource_path.as_str().is_empty() {
      return None;
    }
    let bytes = loader_context.context.fs.read(&resource_path).await.ok()?;
    let mut hasher = RspackHasher::new(&HashFunction::Xxhash64);
    hasher.write(&bytes);
    Some(hasher.finish())
  }

  fn cache_key(loader_context: &LoaderContext<RunnerContext>) -> LoaderCacheKey {
    LoaderCacheKey {
      module_identifier: loader_context
        .context
        .module
        .identifier()
        .as_str()
        .to_owned(),
      remaining_request: loader_context.remaining_request().to_string(),
    }
  }

  fn record_pitch_data(
    loader_context: &mut LoaderContext<RunnerContext>,
    key: LoaderCacheKey,
    resource_hash: u64,
  ) {
    let data = PitchData {
      key,
      resource_hash,
      diagnostics_len: loader_context.diagnostics.len(),
      file_dependencies: loader_context.file_dependencies.clone(),
      context_dependencies: loader_context.context_dependencies.clone(),
      missing_dependencies: loader_context.missing_dependencies.clone(),
      build_dependencies: loader_context.build_dependencies.clone(),
    };
    let index = loader_context.loader_index as usize;
    loader_context.loader_items[index].set_data(
      serde_json::to_value(data).expect("cache loader pitch data should be serializable"),
    );
  }
}

#[async_trait]
#[cacheable_dyn]
impl Loader<RunnerContext> for CacheLoader {
  fn identifier(&self) -> rspack_collections::Identifier {
    INTERNAL_CACHE_LOADER_IDENTIFIER.into()
  }

  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(resource_hash) = Self::resource_hash(loader_context).await else {
      return Ok(());
    };
    let key = Self::cache_key(loader_context);

    let entry = self
      .cache
      .entries
      .get(&key)
      .filter(|entry| entry.resource_hash == resource_hash)
      .map(|entry| entry.value().clone());
    if let Some(entry) = entry {
      let source_map = entry.source_map.map(|source_map| {
        SourceMap::from_json(source_map)
          .expect("source map serialized by the cache loader should be valid")
      });
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

    self.cache.entries.remove(&key);
    Self::record_pitch_data(loader_context, key, resource_hash);
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let pitch_data =
      serde_json::from_value::<PitchData>(loader_context.current_loader().data().clone()).ok();

    if let Some(pitch_data) = pitch_data
      && loader_context.cacheable
      && loader_context.diagnostics.len() == pitch_data.diagnostics_len
    {
      self.cache.entries.insert(
        pitch_data.key,
        LoaderCacheEntry {
          resource_hash: pitch_data.resource_hash,
          content: loader_context.content().cloned(),
          source_map: loader_context.source_map().map(SourceMap::to_json),
          additional_data: loader_context.additional_data().cloned(),
          file_dependencies: dependency_delta(
            &pitch_data.file_dependencies,
            &loader_context.file_dependencies,
          ),
          context_dependencies: dependency_delta(
            &pitch_data.context_dependencies,
            &loader_context.context_dependencies,
          ),
          missing_dependencies: dependency_delta(
            &pitch_data.missing_dependencies,
            &loader_context.missing_dependencies,
          ),
          build_dependencies: dependency_delta(
            &pitch_data.build_dependencies,
            &loader_context.build_dependencies,
          ),
        },
      );
    }

    loader_context.current_loader().set_finish_called();
    Ok(())
  }
}

#[plugin]
#[derive(Debug)]
#[doc(hidden)]
pub struct LoaderCachePlugin {
  cache: Arc<LoaderCache>,
}

impl LoaderCachePlugin {
  #[doc(hidden)]
  pub fn new() -> Self {
    Self::new_inner(Arc::new(LoaderCache::default()))
  }
}

impl Default for LoaderCachePlugin {
  fn default() -> Self {
    Self::new()
  }
}

#[plugin_hook(NormalModuleFactoryCreateLoaderCache for LoaderCachePlugin)]
async fn create_loader_cache(&self, _loader: &ModuleRuleUseLoader) -> Result<Option<BoxLoader>> {
  Ok(Some(Arc::new(CacheLoader::new(self.cache.clone()))))
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
  use std::path::PathBuf;

  use rustc_hash::FxHashSet;

  use super::{LoaderCachePlugin, dependency_delta, replay_dependency_delta};

  fn dependencies(values: &[&str]) -> FxHashSet<PathBuf> {
    values.iter().map(PathBuf::from).collect()
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
  fn plugin_instances_have_isolated_caches() {
    let first = LoaderCachePlugin::new();
    let second = LoaderCachePlugin::new();

    assert!(!std::sync::Arc::ptr_eq(&first.cache, &second.cache));
  }
}

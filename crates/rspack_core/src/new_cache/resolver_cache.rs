use std::{
  future::Future,
  hash::{Hash, Hasher},
  path::Path,
  sync::Arc,
};

use rspack_cacheable::cacheable;
use rspack_util::time::current_time;
use rustc_hash::FxHasher;

use super::{CacheFacade, CacheValue, FileSystemInfo, Snapshot, SnapshotValidationResult};
use crate::{
  CacheCount, Logger, ResolveDependencies, ResolveInnerError, ResolveResult,
  SnapshotStrategyOptions,
};

#[cacheable]
struct CachedResolution {
  result: ResolveResult,
  dependencies: ResolveDependencies,
  snapshot: Snapshot,
}

/// Cloning shares cache, filesystem-info and statistics handles within one compilation.
#[derive(Debug, Clone)]
pub struct ResolverCache {
  cache: CacheFacade,
  file_system_info: FileSystemInfo,
  strategy: SnapshotStrategyOptions,
  counter: Arc<CacheCount>,
}

impl ResolverCache {
  pub(crate) fn new(
    cache: CacheFacade,
    file_system_info: FileSystemInfo,
    strategy: SnapshotStrategyOptions,
    logger: &impl Logger,
  ) -> Self {
    Self {
      cache,
      file_system_info,
      strategy,
      counter: Arc::new(logger.cache("resolver cache")),
    }
  }

  pub(crate) fn child(&self, name: &str) -> Self {
    Self {
      cache: self.cache.get_child_cache(name),
      file_system_info: self.file_system_info.clone(),
      strategy: self.strategy,
      counter: Arc::clone(&self.counter),
    }
  }

  pub(crate) async fn use_cache<G, F>(
    &self,
    options_key: u64,
    path: &Path,
    request: &str,
    generator: G,
  ) -> (
    Result<ResolveResult, ResolveInnerError>,
    ResolveDependencies,
  )
  where
    G: FnOnce() -> F,
    F: Future<
      Output = (
        Result<ResolveResult, ResolveInnerError>,
        ResolveDependencies,
      ),
    >,
  {
    let mut hasher = FxHasher::default();
    (options_key, path, request).hash(&mut hasher);
    let item = self
      .cache
      .get_item_cache(&format!("{:016x}", hasher.finish()), None);
    if let Some(cached) = item.get::<CachedResolution>()
      && matches!(
        self
          .file_system_info
          .check_snapshot_valid(&cached.snapshot)
          .await,
        Ok(SnapshotValidationResult::Valid)
      )
    {
      self.counter.hit();
      return (Ok(cached.result.clone()), cached.dependencies.clone());
    }
    self.counter.miss();

    let start_time = current_time();
    let (result, dependencies) = generator().await;
    if let Ok(result) = &result
      && let Ok(snapshot) = self
        .file_system_info
        .create_snapshot(
          Some(start_time),
          &dependencies.file_dependencies,
          &Default::default(),
          &dependencies.missing_dependencies,
          self.strategy,
        )
        .await
    {
      item.store(CacheValue::new(CachedResolution {
        result: result.clone(),
        dependencies: dependencies.clone(),
        snapshot,
      }));
    }
    (result, dependencies)
  }

  pub(crate) fn log(&self, logger: &impl Logger) {
    logger.cache_end(self.counter.as_ref());
  }
}

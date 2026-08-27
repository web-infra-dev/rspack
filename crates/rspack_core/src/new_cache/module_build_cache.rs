use std::sync::{Arc, Mutex};

use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hash::HashFunction;

use super::{
  CacheFacade, CacheValue,
  snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult},
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildInfo, BuildResult, CompilationLogger,
  CompilationLogging, Module, NormalModuleBuildState, OptimizationBailoutItem, ValueCacheVersions,
  cache::{CacheCodec, SnapshotOptions, SnapshotStrategyOptions},
};

pub(crate) const MODULE_BUILD_CACHE_NAME: &str = "Compilation/modules";

type RestoredDependencies = Vec<BoxDependency>;
// AsyncDependenciesBlock is recursive and intentionally stays behind a pointer.
#[allow(clippy::vec_box)]
type RestoredBlocks = Vec<Box<AsyncDependenciesBlock>>;

#[cacheable]
struct CachedBuildResult<'a> {
  dependencies: Vec<OwnedOrRef<'a, BoxDependency>>,
  // AsyncDependenciesBlock is recursive and intentionally stays behind a pointer.
  #[allow(clippy::vec_box)]
  blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
}

impl CachedBuildResult<'_> {
  fn from_build_result(build_result: &BuildResult) -> CachedBuildResult<'_> {
    CachedBuildResult {
      dependencies: build_result.dependencies.iter().map(Into::into).collect(),
      blocks: build_result
        .blocks
        .iter()
        .map(|block| block.as_ref().into())
        .collect(),
    }
  }

  fn into_parts(self) -> (RestoredDependencies, RestoredBlocks) {
    (
      self
        .dependencies
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .collect(),
      self
        .blocks
        .into_iter()
        .map(|block| Box::new(block.into_owned()))
        .collect(),
    )
  }
}

#[cacheable]
#[derive(Debug)]
struct ModuleBuildCacheEntry {
  state: NormalModuleBuildState,
  build_result: Vec<u8>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildCacheEntry {
  fn build_result_parts(&self, codec: &CacheCodec) -> Result<RestoredModuleBuild> {
    let cached: CachedBuildResult<'static> = codec.decode(&self.build_result)?;
    let (dependencies, blocks) = cached.into_parts();
    Ok(RestoredModuleBuild {
      dependencies,
      blocks,
      optimization_bailouts: self.optimization_bailouts.clone(),
    })
  }
}

/// A tombstone replaces an older cacheable entry when the latest build opts out.
#[cacheable]
#[derive(Debug)]
enum ModuleBuildCacheValue {
  Cacheable(ModuleBuildCacheEntry),
  NotCacheable,
}

/// The outer cache stores an already encoded value so serialization failures are
/// reported before `succeedModule`, instead of being deferred to idle flushing.
#[cacheable]
#[derive(Debug)]
struct EncodedModuleBuildCacheValue {
  bytes: Vec<u8>,
}

/// Compilation-local view of webpack's `Compilation/modules` cache.
#[derive(Debug, Clone)]
pub(crate) struct ModuleCache {
  cache: CacheFacade,
  codec: Arc<CacheCodec>,
  file_system_info: FileSystemInfo,
  snapshot_strategy: SnapshotStrategyOptions,
  invalid_modules: Arc<Mutex<IdentifierSet>>,
}

impl ModuleCache {
  pub(crate) fn new(
    cache: CacheFacade,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    logging: CompilationLogging,
    snapshot_options: SnapshotOptions,
    hash_function: HashFunction,
    snapshot_strategy: SnapshotStrategyOptions,
  ) -> Self {
    Self {
      codec: cache.codec(),
      cache,
      file_system_info: FileSystemInfo::new(
        input_filesystem,
        CompilationLogger::new("rspack.ModuleBuildCache".to_string(), logging),
        snapshot_options,
        hash_function,
      ),
      snapshot_strategy,
      invalid_modules: Default::default(),
    }
  }

  pub(crate) fn invalidate(&self, modules: &IdentifierSet) {
    self
      .invalid_modules
      .lock()
      .expect("module cache invalidation lock should not be poisoned")
      .extend(modules.iter().copied());
  }
}

#[derive(Debug)]
pub(crate) struct RestoredModuleBuild {
  dependencies: RestoredDependencies,
  blocks: RestoredBlocks,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl RestoredModuleBuild {
  pub(crate) fn into_build_result(self, module: BoxModule) -> BuildResult {
    BuildResult {
      module,
      dependencies: self.dependencies,
      blocks: self.blocks,
      optimization_bailouts: self.optimization_bailouts,
    }
  }
}

impl ModuleCache {
  /// This is the single validity contract for a restored NormalModule build.
  async fn need_build(
    &self,
    entry: &ModuleBuildCacheEntry,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<bool> {
    if entry.state.need_build(value_cache_versions) {
      return Ok(true);
    }
    let Some(snapshot) = entry.state.snapshot() else {
      return Ok(true);
    };
    Ok(!matches!(
      self.file_system_info.check_snapshot_valid(snapshot).await?,
      SnapshotValidationResult::Valid
    ))
  }

  #[tracing::instrument("Cache::ModuleBuild::restore", skip_all)]
  pub(crate) async fn restore(
    &self,
    module: &mut BoxModule,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<Option<RestoredModuleBuild>> {
    if module.as_normal_module().is_none() {
      return Ok(None);
    }
    if self
      .invalid_modules
      .lock()
      .expect("module cache invalidation lock should not be poisoned")
      .remove(&module.identifier())
    {
      return Ok(None);
    }
    let item_cache = self
      .cache
      .get_item_cache(module.identifier().as_str(), None);
    let Some(encoded) = item_cache.get::<EncodedModuleBuildCacheValue>()? else {
      return Ok(None);
    };
    let value = self.codec.decode::<ModuleBuildCacheValue>(&encoded.bytes)?;
    let ModuleBuildCacheValue::Cacheable(entry) = value else {
      return Ok(None);
    };
    if self.need_build(&entry, value_cache_versions).await? {
      return Ok(None);
    }

    let restored = entry.build_result_parts(&self.codec)?;
    let Some(module) = module.as_normal_module_mut() else {
      return Ok(None);
    };
    module.restore_build_state(&entry.state);
    Ok(Some(restored))
  }

  #[tracing::instrument("Cache::ModuleBuild::store", skip_all)]
  pub(crate) async fn store(&self, build_result: &mut BuildResult, start_time: u64) -> Result<()> {
    let Some(module) = build_result.module.as_normal_module() else {
      return Ok(());
    };
    let cacheable = module.build_info().cacheable;
    let item_cache = self
      .cache
      .get_item_cache(build_result.module.identifier().as_str(), None);
    let value = if cacheable {
      let snapshot = self
        .create_snapshot(module.build_info(), start_time)
        .await?;
      let build_result_bytes = self
        .codec
        .encode(&CachedBuildResult::from_build_result(build_result))?;
      let optimization_bailouts = build_result.optimization_bailouts.clone();
      let module = build_result
        .module
        .as_normal_module_mut()
        .expect("module type should not change while storing build cache");
      module.build_info_mut().snapshot = Some(Box::new(snapshot));
      ModuleBuildCacheValue::Cacheable(ModuleBuildCacheEntry {
        state: module.build_state(),
        build_result: build_result_bytes,
        optimization_bailouts,
      })
    } else {
      build_result
        .module
        .as_normal_module_mut()
        .expect("module type should not change while storing build cache")
        .build_info_mut()
        .snapshot = None;
      ModuleBuildCacheValue::NotCacheable
    };
    let bytes = self.codec.encode(&value)?;
    item_cache.store(CacheValue::new(EncodedModuleBuildCacheValue { bytes }))
  }

  async fn create_snapshot(&self, build_info: &BuildInfo, start_time: u64) -> Result<Snapshot> {
    self
      .file_system_info
      .create_snapshot(
        Some(start_time),
        &build_info.dependencies.file,
        &build_info.dependencies.context,
        &build_info.dependencies.missing,
        self.snapshot_strategy,
      )
      .await
  }
}

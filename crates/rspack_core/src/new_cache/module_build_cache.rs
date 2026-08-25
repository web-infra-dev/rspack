use std::{
  borrow::Cow,
  sync::{Arc, OnceLock},
};

use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_hash::HashFunction;
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::{
  Cache, CacheFacade, CacheValue, IdleFileCache,
  snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult},
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildInfo, BuildResult, Module,
  NormalModuleBuildState, OptimizationBailoutItem, ValueCacheVersions,
  cache::{CacheCodec, SnapshotOptions},
};

const MODULE_BUILD_CACHE_NAME: &str = "Compilation/modules";

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
  snapshot: Snapshot,
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

/// Creates compilation-local module cache views and owns persistent module metadata.
#[derive(Debug)]
pub(crate) struct ModuleCacheFactory {
  cache: CacheFacade,
  codec: Arc<CacheCodec>,
  idle_file_cache: Option<IdleFileCache>,
  dependency_id_restored: OnceLock<()>,
}

impl ModuleCacheFactory {
  pub(crate) fn new(
    cache: Cache,
    codec: Arc<CacheCodec>,
    idle_file_cache: Option<IdleFileCache>,
  ) -> Self {
    Self {
      cache: cache.facade(MODULE_BUILD_CACHE_NAME),
      codec,
      idle_file_cache,
      dependency_id_restored: OnceLock::new(),
    }
  }

  pub(crate) fn create_for_compilation(
    &self,
    input_filesystem: Arc<dyn ReadableFileSystem>,
    snapshot_options: SnapshotOptions,
    hash_function: HashFunction,
  ) -> ModuleCache {
    ModuleCache {
      cache: self.cache.clone(),
      codec: self.codec.clone(),
      file_system_info: FileSystemInfo::new(input_filesystem, snapshot_options, hash_function),
    }
  }

  pub(crate) fn restore_dependency_id(&self) {
    self.dependency_id_restored.get_or_init(|| {
      let Some(file_cache) = &self.idle_file_cache else {
        return;
      };
      let dependency_id = match file_cache.restore_dependency_id() {
        Ok(dependency_id) => dependency_id,
        Err(error) => {
          tracing::warn!("Restoring new cache dependency id failed: {error}");
          return;
        }
      };
      let current = get_current_dependency_id();
      if current < dependency_id {
        set_current_dependency_id(dependency_id);
      }
    });
  }

  pub(crate) fn record_dependency_id(&self, dependency_id: u32) -> Result<()> {
    if let Some(file_cache) = &self.idle_file_cache {
      file_cache.store_dependency_id(dependency_id)
    } else {
      Ok(())
    }
  }
}

/// Compilation-local view of webpack's `Compilation/modules` cache.
#[derive(Debug)]
pub(crate) struct ModuleCache {
  cache: CacheFacade,
  codec: Arc<CacheCodec>,
  file_system_info: FileSystemInfo,
}

impl ModuleCache {
  pub(crate) fn build_cache(&self, value_cache_versions: &ValueCacheVersions) -> ModuleBuildCache {
    ModuleBuildCache {
      cache: self.cache.clone(),
      codec: self.codec.clone(),
      file_system_info: self.file_system_info.clone(),
      value_cache_versions: Arc::new(value_cache_versions.clone()),
    }
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

/// Per-NormalModule build cache aligned with webpack's `Compilation/modules` cache.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  codec: Arc<CacheCodec>,
  file_system_info: FileSystemInfo,
  value_cache_versions: Arc<ValueCacheVersions>,
}

impl ModuleBuildCache {
  #[tracing::instrument("Cache::ModuleBuild::restore", skip_all)]
  pub(crate) async fn restore(
    &self,
    module: &mut BoxModule,
  ) -> Result<Option<RestoredModuleBuild>> {
    if module.as_normal_module().is_none() {
      return Ok(None);
    }
    let item_cache = self
      .cache
      .get_item_cache(module.identifier().as_str(), None);
    let Some(entry) = item_cache.get::<ModuleBuildCacheEntry>()? else {
      return Ok(None);
    };
    if entry.state.need_build(&self.value_cache_versions)
      || !matches!(
        self
          .file_system_info
          .check_snapshot_valid(&entry.snapshot)
          .await?,
        SnapshotValidationResult::Valid
      )
    {
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
  pub(crate) async fn store(&self, build_result: &BuildResult, start_time: u64) -> Result<()> {
    let Some(module) = build_result.module.as_normal_module() else {
      return Ok(());
    };
    let build_info = module.build_info();
    if !build_info.cacheable {
      return Ok(());
    }
    let snapshot = self.create_snapshot(build_info, start_time).await?;
    let entry = ModuleBuildCacheEntry {
      state: module.build_state(),
      snapshot,
      build_result: self
        .codec
        .encode(&CachedBuildResult::from_build_result(build_result))?,
      optimization_bailouts: build_result.optimization_bailouts.clone(),
    };
    self
      .cache
      .get_item_cache(build_result.module.identifier().as_str(), None)
      .store(CacheValue::new(entry))
  }

  async fn create_snapshot(&self, build_info: &BuildInfo, start_time: u64) -> Result<Snapshot> {
    let files = if build_info.build_dependencies.is_empty() {
      Cow::Borrowed(&build_info.file_dependencies)
    } else {
      let mut files = build_info.file_dependencies.clone();
      files.extend(build_info.build_dependencies.iter().cloned());
      Cow::Owned(files)
    };
    self
      .file_system_info
      .create_snapshot(
        Some(start_time),
        &files,
        &build_info.context_dependencies,
        &build_info.missing_dependencies,
        self.file_system_info.module_strategy(),
      )
      .await
  }
}

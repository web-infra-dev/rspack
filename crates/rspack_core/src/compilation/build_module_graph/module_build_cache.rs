use rspack_collections::Identifiable;
use rspack_error::{Result, error};

use crate::{
  BoxModule, BuildResult, CacheOptions, CompilerOptions, FileSystemInfo, ValueCacheVersions,
  cache::{CacheCodec, SnapshotStrategyOptions},
  new_cache::{CacheFacade, CacheValue},
};

/// Persistent cache for completed normal module builds.
///
/// Webpack caches the `Module` itself. Rspack keeps dependencies and blocks in
/// `BuildResult` until they are inserted into the module graph, so caching the
/// complete build result is the equivalent representation. The serialized form
/// also gives each compilation an exclusively owned module while the generic
/// cache continues to expose shared immutable values.
///
/// Context modules are intentionally built fresh until their factory state can
/// be restored without serializing the process-local dependency resolver.
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuildCache {
  cache: CacheFacade,
  codec: CacheCodec,
}

impl ModuleBuildCache {
  pub(crate) fn new(cache: CacheFacade, options: &CompilerOptions) -> Self {
    let project_root = match &options.cache {
      CacheOptions::Persistent(cache_options) if cache_options.portable => {
        Some(options.context.as_path().to_path_buf())
      }
      _ => None,
    };
    Self {
      cache,
      codec: CacheCodec::new(project_root),
    }
  }

  pub(crate) async fn restore(
    &self,
    module: &mut BoxModule,
    file_system_info: &FileSystemInfo,
    value_cache_versions: &ValueCacheVersions,
  ) -> Result<Option<BuildResult>> {
    if module.as_normal_module().is_none() {
      return Ok(None);
    }

    let identifier = module.identifier();
    let Some(bytes) = self.cache.get::<Vec<u8>>(identifier.as_str(), None)? else {
      return Ok(None);
    };
    let mut result = self.codec.decode::<BuildResult>(&bytes)?;
    if result.module.identifier() != identifier {
      return Err(error!(
        "Restored module identifier mismatch: expected {identifier}, got {}",
        result.module.identifier()
      ));
    }
    if result.module.as_normal_module().is_none() {
      return Err(error!(
        "Restored module type mismatch: expected a normal module for {identifier}"
      ));
    }

    result.module.update_cache_module(module);
    let need_build = result
      .module
      .as_normal_module()
      .expect("restored module type was checked above")
      .need_build_with_context(file_system_info, value_cache_versions)
      .await?;
    if need_build {
      *module = result.module;
      return Ok(None);
    }

    Ok(Some(result))
  }

  pub(crate) async fn store(
    &self,
    result: &mut BuildResult,
    file_system_info: &FileSystemInfo,
    build_start_time: u64,
  ) -> Result<()> {
    let module = &mut result.module;
    if module.as_normal_module().is_none() {
      return Ok(());
    }

    if module.build_info().cacheable
      && !module
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.is_error())
    {
      let build_info = module.build_info();
      let snapshot = file_system_info
        .create_snapshot(
          Some(build_start_time),
          &build_info.dependencies.file,
          &build_info.dependencies.context,
          &build_info.dependencies.missing,
          // Rspack does not expose webpack's `snapshot.module` strategy yet.
          SnapshotStrategyOptions::timestamp(),
        )
        .await?;
      module.build_info_mut().snapshot = Some(snapshot);
    }

    let identifier = result.module.identifier();
    let bytes = match self.codec.encode(result) {
      Ok(bytes) => bytes,
      Err(error) => {
        // Match webpack's persistent cache behavior: an item containing
        // process-local state is skipped instead of failing the compilation.
        tracing::debug!(
          module = identifier.as_str(),
          %error,
          "Skipped non-serializable module cache entry"
        );
        return Ok(());
      }
    };
    self
      .cache
      .store(identifier.as_str(), None, CacheValue::new(bytes))
  }
}

use rspack_collections::Identifiable;
use rspack_error::{Result, error};

use super::{Cache, CacheFacade, CacheValue};
use crate::{
  BuildResult, CacheOptions, CompilerOptions, FileSystemInfo, ModuleIdentifier,
  cache::{CacheCodec, SnapshotStrategyOptions},
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
pub(crate) struct ModuleCache {
  cache: CacheFacade,
  codec: CacheCodec,
  enabled: bool,
}

impl ModuleCache {
  pub fn new(cache: Cache, options: &CompilerOptions, is_rebuild: bool) -> Self {
    let project_root = match &options.cache {
      CacheOptions::Persistent(cache_options) if cache_options.portable => {
        Some(options.context.as_path().to_path_buf())
      }
      _ => None,
    };
    Self {
      cache: cache.facade("Compilation/modules"),
      codec: CacheCodec::new(project_root),
      // Incremental make reuses the previous module graph and owns its own
      // invalidation path. Keep that fast path unchanged.
      enabled: options.experiments.new_cache.module && !is_rebuild,
    }
  }

  pub fn restore(&self, identifier: ModuleIdentifier) -> Result<Option<BuildResult>> {
    if !self.enabled {
      return Ok(None);
    }
    let Some(bytes) = self.cache.get::<Vec<u8>>(identifier.as_str(), None)? else {
      return Ok(None);
    };
    let result = self.codec.decode::<BuildResult>(&bytes)?;
    if result.module.identifier() != identifier {
      return Err(error!(
        "Restored module identifier mismatch: expected {identifier}, got {}",
        result.module.identifier()
      ));
    }
    Ok(Some(result))
  }

  pub async fn store(
    &self,
    result: &mut BuildResult,
    file_system_info: &FileSystemInfo,
    build_start_time: u64,
  ) -> Result<()> {
    if !self.enabled {
      return Ok(());
    }

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

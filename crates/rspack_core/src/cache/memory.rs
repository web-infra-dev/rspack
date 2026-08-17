use super::{Cache, persistent::occasion::SourceMapDevToolPluginCacheArtifact};
use crate::Compilation;

/// Process-local build cache.
///
/// Incremental artifacts are owned by `IncrementalArtifacts`; this cache only
/// retains data that is explicitly controlled by the cache option.
#[derive(Debug, Default)]
pub struct MemoryCache {
  source_map_dev_tool_plugin_cache_artifact: Option<SourceMapDevToolPluginCacheArtifact>,
}

#[async_trait::async_trait]
impl Cache for MemoryCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    compilation.is_rebuild
  }

  fn store_hot_cache(&mut self, compilation: &mut Compilation) {
    self.source_map_dev_tool_plugin_cache_artifact =
      compilation.source_map_dev_tool_plugin_cache_artifact.take();
  }

  async fn before_process_assets(&mut self, compilation: &mut Compilation) {
    if compilation.use_source_map_dev_tool_plugin_cache {
      compilation.source_map_dev_tool_plugin_cache_artifact = Some(
        self
          .source_map_dev_tool_plugin_cache_artifact
          .take()
          .unwrap_or_default(),
      );
    }
  }
}

use super::Cache;
use crate::compilation::build_module_graph::BuildModuleGraphArtifact;

/// Disable cache implementation
///
/// Disable cache will clean the corresponding artifact before target step run.
#[derive(Debug)]
pub struct DisableCache;

#[async_trait::async_trait]
impl Cache for DisableCache {
  async fn before_build_module_graph(&mut self, _make_artifact: &mut BuildModuleGraphArtifact) {
    // do nothing just don't use any cache
  }
}

use super::Cache;
use crate::{compilation::build_module_graph::BuildModuleGraphArtifact, incremental::Incremental};

/// Disable cache implementation
///
/// Disable cache will clean the corresponding artifact before target step run.
#[derive(Debug)]
pub(super) struct DisableCache;

#[async_trait::async_trait]
impl Cache for DisableCache {
  async fn before_build_module_graph(
    &mut self,
    make_artifact: &mut BuildModuleGraphArtifact,
    _incremental: &Incremental,
  ) {
    *make_artifact = Default::default();
  }
}

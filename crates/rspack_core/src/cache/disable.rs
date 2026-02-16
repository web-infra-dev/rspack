use super::Cache;
use crate::{Compilation, compilation::build_module_graph::BuildModuleGraphArtifact};

/// Disable cache implementation
///
/// Disable cache will clean the corresponding artifact before target step run.
#[derive(Debug)]
pub struct DisableCache;

#[async_trait::async_trait]
impl Cache for DisableCache {
  async fn before_build_module_graph(&mut self, compilation: &mut Compilation) {
    *compilation.build_module_graph_artifact = BuildModuleGraphArtifact::new();
  }
}

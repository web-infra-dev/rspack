use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct OptimizeChunksPass;

#[async_trait]
impl PassExt for OptimizeChunksPass {
  fn name(&self) -> &'static str {
    "optimize chunks"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let mut diagnostics = vec![];
    let mut build_chunk_graph_artifact =
      std::mem::take(&mut compilation.build_chunk_graph_artifact);
    while matches!(
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .optimize_chunks
        .call(
          compilation,
          &mut build_chunk_graph_artifact,
          &mut diagnostics,
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunks"))?,
      Some(true)
    ) {}
    compilation.build_chunk_graph_artifact = build_chunk_graph_artifact;
    compilation.extend_diagnostics(diagnostics);

    Ok(())
  }
}

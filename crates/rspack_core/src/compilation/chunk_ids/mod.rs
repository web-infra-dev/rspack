use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct ChunkIdsPass;

#[async_trait]
impl PassExt for ChunkIdsPass {
  fn name(&self) -> &'static str {
    "chunk ids"
  }

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::CHUNK_IDS
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    // Check if CHUNK_IDS pass is disabled, and clear artifact if needed
    if !compilation
      .incremental
      .passes_enabled(IncrementalPasses::CHUNK_IDS)
    {
      compilation.named_chunk_ids_artifact.clear();
    }

    let mut diagnostics = vec![];
    let mut chunk_by_ukey = mem::take(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
    let mut named_chunk_ids_artifact = compilation.named_chunk_ids_artifact.steal();
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .chunk_ids
      .call(
        compilation,
        &mut chunk_by_ukey,
        &mut named_chunk_ids_artifact,
        &mut diagnostics,
      )
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.chunkIds"))?;
    compilation.build_chunk_graph_artifact.chunk_by_ukey = chunk_by_ukey;
    compilation.named_chunk_ids_artifact = named_chunk_ids_artifact.into();
    compilation.extend_diagnostics(diagnostics);

    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .after_optimize_chunk_ids
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterOptimizeChunkIds"))?;

    Ok(())
  }
}

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

    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .chunk_ids
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.chunkIds"))?;
    Ok(())
  }
}

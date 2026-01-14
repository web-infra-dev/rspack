use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn chunk_ids_pass(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("chunk ids");

    // Check if CHUNK_IDS pass is disabled, and clear artifact if needed
    if !self
      .incremental
      .passes_enabled(IncrementalPasses::CHUNK_IDS)
    {
      self.named_chunk_ids_artifact.clear();
    }

    let mut diagnostics = vec![];
    let mut chunk_by_ukey = mem::take(&mut self.chunk_by_ukey);
    let mut named_chunk_ids_artifact = mem::take(&mut self.named_chunk_ids_artifact);
    plugin_driver
      .compilation_hooks
      .chunk_ids
      .call(
        self,
        &mut chunk_by_ukey,
        &mut named_chunk_ids_artifact,
        &mut diagnostics,
      )
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.chunkIds"))?;
    self.chunk_by_ukey = chunk_by_ukey;
    self.named_chunk_ids_artifact = named_chunk_ids_artifact;
    self.extend_diagnostics(diagnostics);
    logger.time_end(start);
    Ok(())
  }
}

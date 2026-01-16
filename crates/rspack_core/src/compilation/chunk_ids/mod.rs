use super::*;
use crate::logger::Logger;

pub async fn chunk_ids_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("chunk ids");

  // Check if CHUNK_IDS pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::CHUNK_IDS)
  {
    compilation.named_chunk_ids_artifact.clear();
  }

  let mut diagnostics = vec![];
  let mut chunk_by_ukey = mem::take(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
  let mut named_chunk_ids_artifact = mem::take(&mut compilation.named_chunk_ids_artifact);
  plugin_driver
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
  compilation.named_chunk_ids_artifact = named_chunk_ids_artifact;
  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}

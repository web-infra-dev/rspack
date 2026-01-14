use std::mem;

use rspack_error::Result;

use crate::{Compilation, Logger, SharedPluginDriver, reset_artifact_if_passes_disabled};

pub async fn chunk_ids(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("chunk ids");

  // Check if CHUNK_IDS pass is disabled, and reset artifact state if needed
  reset_artifact_if_passes_disabled(
    &compilation.incremental,
    &mut compilation.named_chunk_ids_artifact,
  );

  let mut diagnostics = vec![];
  let mut chunk_by_ukey = mem::take(&mut compilation.chunk_by_ukey);
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
  compilation.chunk_by_ukey = chunk_by_ukey;
  compilation.named_chunk_ids_artifact = named_chunk_ids_artifact;
  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}

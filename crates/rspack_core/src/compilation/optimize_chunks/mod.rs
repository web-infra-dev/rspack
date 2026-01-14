use super::*;

impl Compilation {
  pub async fn optimize_chunks_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_chunks
        .call(self)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunks"))?,
      Some(true)
    ) {}
    Ok(())
  }
}

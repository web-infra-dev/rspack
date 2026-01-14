use super::*;

impl Compilation {
  pub async fn optimize_chunk_modules_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .optimize_chunk_modules
      .call(self)
      .await
      .map(|_| ())
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"))
  }
}

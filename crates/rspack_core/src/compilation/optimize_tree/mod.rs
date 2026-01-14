use super::*;

impl Compilation {
  pub async fn optimize_tree_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .optimize_tree
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeTree"))
  }
}

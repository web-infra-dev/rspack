use super::*;

pub async fn optimize_tree_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  plugin_driver
    .compilation_hooks
    .optimize_tree
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeTree"))
}

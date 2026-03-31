use rspack_error::Result;

use crate::{Compilation, SharedPluginDriver, cache::Cache};

pub async fn make_hook_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
  cache: &mut dyn Cache,
) -> Result<()> {
  cache
    .before_build_module_graph(&mut compilation.build_module_graph_artifact)
    .await;

  plugin_driver.compiler_hooks.make.call(compilation).await?;

  Ok(())
}

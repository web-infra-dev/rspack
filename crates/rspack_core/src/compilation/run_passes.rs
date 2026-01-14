use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn run_passes(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    self
      .optimize_dependencies_pass(plugin_driver.clone())
      .await?;

    let logger = self.get_logger("rspack.Compilation");

    let create_chunks_start = logger.time("create chunks");
    self.build_chunk_graph_pass().await?;
    self
      .optimize_modules_pass(plugin_driver.clone())
      .await?;
    self
      .optimize_chunks_pass(plugin_driver.clone())
      .await?;
    logger.time_end(create_chunks_start);

    let optimize_start = logger.time("optimize");
    self
      .optimize_tree_pass(plugin_driver.clone())
      .await?;
    self
      .optimize_chunk_modules_pass(plugin_driver.clone())
      .await?;
    logger.time_end(optimize_start);

    self.module_ids_pass(plugin_driver.clone()).await?;
    self.chunk_ids_pass(plugin_driver.clone()).await?;
    self.assign_runtime_ids();

    self
      .optimize_code_generation_pass(plugin_driver.clone())
      .await?;
    self.create_module_hashes_pass().await?;
    self.code_generation_pass(plugin_driver.clone()).await?;
    self
      .runtime_requirements_pass(plugin_driver.clone())
      .await?;
    self.create_hash_pass(plugin_driver.clone()).await?;
    self
      .create_module_assets_pass(plugin_driver.clone())
      .await?;
    self
      .create_chunk_assets_pass(plugin_driver.clone())
      .await?;
    self.process_assets_pass(plugin_driver.clone()).await?;
    self.after_seal_pass(plugin_driver).await?;

    Ok(())
  }
}

use super::{
  after_seal::after_seal_pass, assign_runtime_ids::assign_runtime_ids,
  build_chunk_graph::pass::build_chunk_graph_pass, build_module_graph::build_module_graph_pass,
  chunk_ids::chunk_ids_pass, code_generation::code_generation_pass,
  create_chunk_assets::create_chunk_assets_pass, create_hash::create_hash_pass,
  create_module_assets::create_module_assets_pass, create_module_hashes::create_module_hashes_pass,
  finish_make::finish_make_pass, finish_module_graph::finish_module_graph_pass,
  finish_modules::finish_modules_pass, make::make_hook_pass, module_ids::module_ids_pass,
  optimize_chunk_modules::optimize_chunk_modules_pass, optimize_chunks::optimize_chunks_pass,
  optimize_code_generation::optimize_code_generation_pass,
  optimize_dependencies::optimize_dependencies_pass, optimize_modules::optimize_modules_pass,
  optimize_tree::optimize_tree_pass, process_assets::process_assets_pass,
  runtime_requirements::runtime_requirements_pass, seal::seal_pass, *,
};
use crate::{Compilation, SharedPluginDriver, cache::Cache};

impl Compilation {
  pub async fn run_passes(
    &mut self,
    plugin_driver: SharedPluginDriver,
    cache: &mut dyn Cache,
  ) -> Result<()> {
    make_hook_pass(self, plugin_driver.clone(), cache).await?;
    build_module_graph_pass(self).await?;
    finish_make_pass(self, plugin_driver.clone()).await?;
    finish_module_graph_pass(self, cache).await?;
    finish_modules_pass(self).await?;
    // This is the end of first pass of build module graph which will be recovered for next compilation
    // add a checkpoint here since we may modify module graph later in incremental compilation
    // and we can recover to this checkpoint in the future
    if self
      .incremental
      .passes_enabled(IncrementalPasses::BUILD_MODULE_GRAPH)
    {
      self.build_module_graph_artifact.module_graph.checkpoint();
    }
    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.freeze();
    }

    seal_pass(self, plugin_driver.clone()).await?;

    optimize_dependencies_pass(self, plugin_driver.clone()).await?;

    build_chunk_graph_pass(self).await?;
    optimize_modules_pass(self, plugin_driver.clone()).await?;
    optimize_chunks_pass(self, plugin_driver.clone()).await?;

    optimize_tree_pass(self, plugin_driver.clone()).await?;
    optimize_chunk_modules_pass(self, plugin_driver.clone()).await?;

    module_ids_pass(self, plugin_driver.clone()).await?;
    chunk_ids_pass(self, plugin_driver.clone()).await?;
    assign_runtime_ids(self);

    optimize_code_generation_pass(self, plugin_driver.clone()).await?;
    create_module_hashes_pass(self).await?;
    code_generation_pass(self, plugin_driver.clone()).await?;
    runtime_requirements_pass(self, plugin_driver.clone()).await?;
    create_hash_pass(self, plugin_driver.clone()).await?;
    create_module_assets_pass(self, plugin_driver.clone()).await?;
    create_chunk_assets_pass(self, plugin_driver.clone()).await?;
    process_assets_pass(self, plugin_driver.clone()).await?;
    after_seal_pass(self, plugin_driver).await?;

    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.unfreeze();
    }
    Ok(())
  }
}

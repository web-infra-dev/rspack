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
    // #region Build Module Graph First Pass
    cache
      .before_build_module_graph(&mut self.build_module_graph_artifact, &self.incremental)
      .await;
    make_hook_pass(self, plugin_driver.clone()).await?;
    build_module_graph_pass(self).await?;
    finish_make_pass(self, plugin_driver.clone()).await?;

    finish_module_graph_pass(self).await?;

    cache
      .after_build_module_graph(&self.build_module_graph_artifact)
      .await;
    if self
      .incremental
      .passes_enabled(IncrementalPasses::BUILD_MODULE_GRAPH)
    {
      self.build_module_graph_artifact.module_graph.checkpoint();
    }
    // #endregion Build Module Graph First Pass Finished here and will be use to recover for next compilation

    // FINISH_MODULES pass
    cache.before_finish_modules(self).await;
    // finish_modules will set exports_info for build_module_graph_artifact so we have to put it here
    finish_modules_pass(self).await?;
    cache.after_finish_modules(self).await;

    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.freeze();
    }

    seal_pass(self, plugin_driver.clone()).await?;

    // OPTIMIZE_DEPENDENCIES pass
    cache.before_optimize_dependencies(self).await;
    optimize_dependencies_pass(self, plugin_driver.clone()).await?;
    cache.after_optimize_dependencies(self).await;

    // BUILD_CHUNK_GRAPH pass
    cache.before_build_chunk_graph(self).await;
    build_chunk_graph_pass(self).await?;
    cache.after_build_chunk_graph(self).await;

    optimize_modules_pass(self, plugin_driver.clone()).await?;
    optimize_chunks_pass(self, plugin_driver.clone()).await?;

    optimize_tree_pass(self, plugin_driver.clone()).await?;
    optimize_chunk_modules_pass(self, plugin_driver.clone()).await?;

    // MODULE_IDS pass
    cache.before_module_ids(self).await;
    module_ids_pass(self, plugin_driver.clone()).await?;
    cache.after_module_ids(self).await;

    // CHUNK_IDS pass
    cache.before_chunk_ids(self).await;
    chunk_ids_pass(self, plugin_driver.clone()).await?;
    cache.after_chunk_ids(self).await;

    assign_runtime_ids(self);

    optimize_code_generation_pass(self, plugin_driver.clone()).await?;

    // MODULES_HASHES pass
    cache.before_modules_hashes(self).await;
    create_module_hashes_pass(self).await?;
    cache.after_modules_hashes(self).await;

    // MODULES_CODEGEN pass
    cache.before_modules_codegen(self).await;
    code_generation_pass(self, plugin_driver.clone()).await?;
    cache.after_modules_codegen(self).await;

    // MODULES_RUNTIME_REQUIREMENTS and CHUNKS_RUNTIME_REQUIREMENTS passes
    cache.before_modules_runtime_requirements(self).await;
    cache.before_chunks_runtime_requirements(self).await;
    runtime_requirements_pass(self, plugin_driver.clone()).await?;
    cache.after_modules_runtime_requirements(self).await;
    cache.after_chunks_runtime_requirements(self).await;

    // CHUNKS_HASHES pass
    cache.before_chunks_hashes(self).await;
    create_hash_pass(self, plugin_driver.clone()).await?;
    cache.after_chunks_hashes(self).await;

    create_module_assets_pass(self, plugin_driver.clone()).await?;

    // CHUNK_ASSET pass
    cache.before_chunk_asset(self).await;
    create_chunk_assets_pass(self, plugin_driver.clone()).await?;
    cache.after_chunk_asset(self).await;

    process_assets_pass(self, plugin_driver.clone()).await?;
    after_seal_pass(self, plugin_driver).await?;

    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.unfreeze();
    }
    Ok(())
  }
}

use super::{
  after_seal::after_seal_pass, assign_runtime_ids::assign_runtime_ids,
  build_chunk_graph::pass::build_chunk_graph_pass, chunk_ids::chunk_ids_pass,
  code_generation::code_generation_pass, create_chunk_assets::create_chunk_assets_pass,
  create_hash::create_hash_pass, create_module_assets::create_module_assets_pass,
  create_module_hashes::create_module_hashes_pass, module_ids::module_ids_pass,
  optimize_chunk_modules::optimize_chunk_modules_pass, optimize_chunks::optimize_chunks_pass,
  optimize_code_generation::optimize_code_generation_pass,
  optimize_dependencies::optimize_dependencies_pass, optimize_modules::optimize_modules_pass,
  optimize_tree::optimize_tree_pass, process_assets::process_assets_pass,
  runtime_requirements::runtime_requirements_pass, *,
};
use crate::{
  Compilation, SharedPluginDriver, cache::Cache, incremental::IncrementalPasses, logger::Logger,
};

impl Compilation {
  pub async fn run_passes(
    &mut self,
    plugin_driver: SharedPluginDriver,
    cache: &mut dyn Cache,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compiler");

    let make_start = logger.time("make");
    let make_hook_start = logger.time("make hook");
    cache
      .before_build_module_graph(&mut self.build_module_graph_artifact)
      .await;

    plugin_driver.compiler_hooks.make.call(self).await?;
    logger.time_end(make_hook_start);
    self.build_module_graph().await?;
    logger.time_end(make_start);

    let start = logger.time("finish make hook");
    plugin_driver.compiler_hooks.finish_make.call(self).await?;
    logger.time_end(start);

    let start = logger.time("finish compilation");
    self.finish_build_module_graph().await?;
    cache
      .after_build_module_graph(&self.build_module_graph_artifact)
      .await;
    logger.time_end(start);

    let dependencies_diagnostics_artifact = self.dependencies_diagnostics_artifact.clone();
    let async_modules_artifact = self.async_modules_artifact.clone();
    let diagnostics = self
      .collect_build_module_graph_effects(
        &mut dependencies_diagnostics_artifact.borrow_mut(),
        &mut async_modules_artifact.borrow_mut(),
      )
      .await?;
    self.extend_diagnostics(diagnostics);

    // add a checkpoint here since we may modify module graph later in incremental compilation
    // and we can recover to this checkpoint in the future
    if self.incremental.passes_enabled(IncrementalPasses::MAKE) {
      self.build_module_graph_artifact.module_graph.checkpoint();
    }

    if !self.options.mode.is_development() {
      self.module_static_cache_artifact.freeze();
    }

    #[cfg(feature = "debug_tool")]
    {
      use rspack_util::debug_tool::wait_for_signal;
      wait_for_signal("seal compilation");
    }
    let logger = self.get_logger("rspack.Compilation");
    // https://github.com/webpack/webpack/blob/main/lib/Compilation.js#L2809
    plugin_driver
      .compilation_hooks
      .seal
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.seal"))?;

    optimize_dependencies_pass(self, plugin_driver.clone()).await?;

    let create_chunks_start = logger.time("create chunks");
    build_chunk_graph_pass(self).await?;
    optimize_modules_pass(self, plugin_driver.clone()).await?;
    optimize_chunks_pass(self, plugin_driver.clone()).await?;
    logger.time_end(create_chunks_start);

    let optimize_start = logger.time("optimize");
    optimize_tree_pass(self, plugin_driver.clone()).await?;
    optimize_chunk_modules_pass(self, plugin_driver.clone()).await?;
    logger.time_end(optimize_start);

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

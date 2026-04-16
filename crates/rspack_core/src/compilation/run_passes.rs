use super::{
  after_process_assets::AfterProcessAssetsPass, after_seal::AfterSealPass,
  assign_runtime_ids::AssignRuntimeIdsPass, build_chunk_graph::pass::BuildChunkGraphPass,
  build_module_graph::build_module_graph_pass, chunk_ids::ChunkIdsPass,
  code_generation::CodeGenerationPass, create_chunk_assets::CreateChunkAssetsPass,
  create_hash::CreateHashPass, create_module_assets::CreateModuleAssetsPass,
  create_module_hashes::CreateModuleHashesPass, finish_make::finish_make_pass,
  finish_module_graph::finish_module_graph_pass, finish_modules::FinishModulesPhasePass,
  make::make_hook_pass, module_ids::ModuleIdsPass,
  optimize_chunk_modules::OptimizeChunkModulesPass, optimize_chunks::OptimizeChunksPass,
  optimize_code_generation::OptimizeCodeGenerationPass,
  optimize_dependencies::OptimizeDependenciesPass, optimize_modules::OptimizeModulesPass,
  optimize_tree::OptimizeTreePass, pass::PassExt, process_assets::ProcessAssetsPass,
  runtime_requirements::RuntimeRequirementsPass, seal::SealPass, *,
};
use crate::{Compilation, SharedPluginDriver, cache::Cache, logger::Logger};

impl Compilation {
  pub async fn run_passes(
    &mut self,
    plugin_driver: SharedPluginDriver,
    cache: &mut dyn Cache,
  ) -> Result<()> {
    let compilation_logger = self.get_logger("rspack.Compilation");
    let logger = self.get_logger("rspack.Compiler");
    let finish_modules_phase = FinishModulesPhasePass;
    let seal_passes: Vec<Box<dyn PassExt>> = vec![
      Box::new(SealPass),
      Box::new(OptimizeDependenciesPass),
      Box::new(BuildChunkGraphPass),
      Box::new(OptimizeModulesPass),
      Box::new(OptimizeChunksPass),
      Box::new(OptimizeTreePass),
      Box::new(OptimizeChunkModulesPass),
      Box::new(ModuleIdsPass),
      Box::new(ChunkIdsPass),
      Box::new(AssignRuntimeIdsPass),
      Box::new(OptimizeCodeGenerationPass),
      Box::new(CreateModuleHashesPass),
      Box::new(CodeGenerationPass),
      Box::new(RuntimeRequirementsPass),
      Box::new(CreateHashPass),
      Box::new(CreateModuleAssetsPass),
      Box::new(CreateChunkAssetsPass),
      Box::new(ProcessAssetsPass),
      Box::new(AfterProcessAssetsPass),
      Box::new(AfterSealPass),
    ];
    if !self.options.mode.is_development() {
      self.module_static_cache.enable_new_cache();
    }

    let build_module_graph_start = compilation_logger.time("build module graph");
    cache.before_build_module_graph(self).await;

    let mut finish_compilation_start = None;
    let build_module_graph_result: Result<()> = async {
      let start = logger.time("make hook");
      make_hook_pass(self, plugin_driver.clone()).await?;
      build_module_graph_pass(self).await?;
      logger.time_end(start);

      finish_make_pass(self, plugin_driver).await?;
      finish_compilation_start = Some(logger.time("finish compilation"));
      finish_module_graph_pass(self).await?;

      use crate::incremental::IncrementalPasses;
      if self
        .incremental
        .passes_enabled(IncrementalPasses::BUILD_MODULE_GRAPH)
      {
        self.build_module_graph_artifact.module_graph.checkpoint();
      }

      Ok(())
    }
    .await;
    if build_module_graph_result.is_ok() {
      cache.after_build_module_graph(self).await;
    }
    compilation_logger.time_end(build_module_graph_start);
    build_module_graph_result?;

    let finish_modules_result = finish_modules_phase.run(self, cache).await;
    if let (Ok(()), Some(start)) = (&finish_modules_result, finish_compilation_start.take()) {
      logger.time_end(start);
    }
    finish_modules_result?;

    let seal_compilation_start = logger.time("seal compilation");
    for pass in &seal_passes {
      pass.run(self, cache).await?;
    }
    logger.time_end(seal_compilation_start);

    if !self.options.mode.is_development() {
      self.module_static_cache.disable_cache();
    }

    Ok(())
  }
}

use super::{
  after_process_assets::AfterProcessAssetsPass, after_seal::AfterSealPass,
  assign_runtime_ids::AssignRuntimeIdsPass, build_chunk_graph::pass::BuildChunkGraphPass,
  build_module_graph::pass::BuildModuleGraphPhasePass, chunk_ids::ChunkIdsPass,
  code_generation::CodeGenerationPass, create_chunk_assets::CreateChunkAssetsPass,
  create_hash::CreateHashPass, create_module_assets::CreateModuleAssetsPass,
  create_module_hashes::CreateModuleHashesPass, finish_modules::FinishModulesPhasePass,
  module_ids::ModuleIdsPass, optimize_chunk_modules::OptimizeChunkModulesPass,
  optimize_chunks::OptimizeChunksPass, optimize_code_generation::OptimizeCodeGenerationPass,
  optimize_dependencies::OptimizeDependenciesPass, optimize_modules::OptimizeModulesPass,
  optimize_tree::OptimizeTreePass, pass::PassExt, process_assets::ProcessAssetsPass,
  runtime_requirements::RuntimeRequirementsPass, seal::SealPass, *,
};
use crate::{Compilation, SharedPluginDriver, cache::Cache};

impl Compilation {
  pub async fn run_passes(
    &mut self,
    _plugin_driver: SharedPluginDriver,
    cache: &mut dyn Cache,
  ) -> Result<()> {
    let passes: Vec<Box<dyn PassExt>> = vec![
      Box::new(BuildModuleGraphPhasePass),
      Box::new(FinishModulesPhasePass),
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

    for pass in &passes {
      pass.run(self, cache).await?;
    }
    if !self.options.mode.is_development() {
      self.module_static_cache.disable_cache();
    }

    Ok(())
  }
}

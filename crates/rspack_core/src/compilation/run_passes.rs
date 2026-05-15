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
    self.module_static_cache.enable_new_cache();

    BuildModuleGraphPhasePass.run(self, cache).await?;
    FinishModulesPhasePass.run(self, cache).await?;
    SealPass.run(self, cache).await?;
    OptimizeDependenciesPass.run(self, cache).await?;
    BuildChunkGraphPass.run(self, cache).await?;
    OptimizeModulesPass.run(self, cache).await?;
    OptimizeChunksPass.run(self, cache).await?;
    OptimizeTreePass.run(self, cache).await?;
    OptimizeChunkModulesPass.run(self, cache).await?;
    ModuleIdsPass.run(self, cache).await?;
    ChunkIdsPass.run(self, cache).await?;
    AssignRuntimeIdsPass.run(self, cache).await?;
    OptimizeCodeGenerationPass.run(self, cache).await?;
    CreateModuleHashesPass.run(self, cache).await?;
    CodeGenerationPass.run(self, cache).await?;
    RuntimeRequirementsPass.run(self, cache).await?;
    CreateHashPass.run(self, cache).await?;
    CreateModuleAssetsPass.run(self, cache).await?;
    CreateChunkAssetsPass.run(self, cache).await?;
    ProcessAssetsPass.run(self, cache).await?;
    AfterProcessAssetsPass.run(self, cache).await?;
    AfterSealPass.run(self, cache).await?;

    self.module_static_cache.disable_cache();

    Ok(())
  }
}

use std::sync::{Arc, RwLock};

/// rust support hooks
#[derive(PartialEq)]
pub enum Hook {
  Make,
  FinishMake,
  BuildModule,
  Compilation,
  ThisCompilation,
  ProcessAssetsStageAdditional,
  ProcessAssetsStagePreProcess,
  ProcessAssetsStageAdditions,
  ProcessAssetsStageNone,
  ProcessAssetsStageOptimizeInline,
  ProcessAssetsStageSummarize,
  ProcessAssetsStageOptimizeHash,
  ProcessAssetsStageReport,
  Emit,
  AssetEmitted,
  AfterEmit,
  OptimizeChunkModules,
  BeforeCompile,
  AfterCompile,
  FinishModules,
  OptimizeModules,
  /// webpack `compilation.hooks.chunkAsset`
  ChunkAsset,
  NormalModuleFactoryResolveForScheme,
  AfterResolve,
  BeforeResolve,
  SucceedModule,
  StillValidModule,
}

impl From<String> for Hook {
  fn from(s: String) -> Self {
    match s.as_str() {
      "make" => Hook::Make,
      "finishMake" => Hook::FinishMake,
      "buildModule" => Hook::BuildModule,
      "compilation" => Hook::Compilation,
      "thisCompilation" => Hook::ThisCompilation,
      "processAssetsStageAdditional" => Hook::ProcessAssetsStageAdditional,
      "processAssetsStagePreProcess" => Hook::ProcessAssetsStagePreProcess,
      "processAssetsStageAdditions" => Hook::ProcessAssetsStageAdditions,
      "processAssetsStageNone" => Hook::ProcessAssetsStageNone,
      "processAssetsStageOptimizeInline" => Hook::ProcessAssetsStageOptimizeInline,
      "processAssetsStageSummarize" => Hook::ProcessAssetsStageSummarize,
      "processAssetsStageOptimizeHash" => Hook::ProcessAssetsStageOptimizeHash,
      "processAssetsStageReport" => Hook::ProcessAssetsStageReport,
      "emit" => Hook::Emit,
      "assetEmitted" => Hook::AssetEmitted,
      "afterEmit" => Hook::AfterEmit,
      "optimizeChunkModules" => Hook::OptimizeChunkModules,
      "beforeCompile" => Hook::BeforeCompile,
      "afterCompile" => Hook::AfterCompile,
      "finishModules" => Hook::FinishModules,
      "optimizeModules" => Hook::OptimizeModules,
      "chunkAsset" => Hook::ChunkAsset,
      "normalModuleFactoryResolveForScheme" => Hook::NormalModuleFactoryResolveForScheme,
      "afterResolve" => Hook::AfterResolve,
      "beforeResolve" => Hook::BeforeResolve,
      "succeedModule" => Hook::SucceedModule,
      "stillValidModule" => Hook::StillValidModule,
      hook_name => panic!("{hook_name} is an invalid hook name"),
    }
  }
}

pub type DisabledHooks = Arc<RwLock<Vec<Hook>>>;

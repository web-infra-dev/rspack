use std::sync::{Arc, RwLock};

/// rust support hooks
#[derive(PartialEq)]
pub enum Hook {
  Make,
  Compilation,
  ThisCompilation,
  ProcessAssetsStageAdditional,
  ProcessAssetsStagePreProcess,
  ProcessAssetsStageNone,
  ProcessAssetsStageOptimizeInline,
  ProcessAssetsStageSummarize,
  ProcessAssetsStageReport,
  Emit,
  AfterEmit,
  OptimizeChunkModules,
}

impl From<String> for Hook {
  fn from(s: String) -> Self {
    match s.as_str() {
      "make" => Hook::Make,
      "compilation" => Hook::Compilation,
      "thisCompilation" => Hook::ThisCompilation,
      "processAssetsStageAdditional" => Hook::ProcessAssetsStageAdditional,
      "processAssetsStagePreProcess" => Hook::ProcessAssetsStagePreProcess,
      "processAssetsStageNone" => Hook::ProcessAssetsStageNone,
      "processAssetsStageOptimizeInline" => Hook::ProcessAssetsStageOptimizeInline,
      "processAssetsStageSummarize" => Hook::ProcessAssetsStageSummarize,
      "processAssetsStageReport" => Hook::ProcessAssetsStageReport,
      "emit" => Hook::Emit,
      "afterEmit" => Hook::AfterEmit,
      "optimizeChunkModules" => Hook::OptimizeChunkModules,
      hook_name => panic!("{hook_name} is an invalid hook name"),
    }
  }
}

pub type DisabledHooks = Arc<RwLock<Vec<Hook>>>;

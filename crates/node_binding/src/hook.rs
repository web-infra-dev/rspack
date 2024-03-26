use std::sync::{Arc, RwLock};

/// rust support hooks
#[derive(PartialEq)]
pub enum Hook {
  Make,
  FinishMake,
  BuildModule,
  ThisCompilation,
  Compilation,
  ProcessAssets,
  AfterProcessAssets,
  Emit,
  AssetEmitted,
  ShouldEmit,
  AfterEmit,
  OptimizeChunkModules,
  FinishModules,
  OptimizeModules,
  AfterOptimizeModules,
  OptimizeTree,
  ChunkAsset,
  ContextModuleFactoryBeforeResolve,
  ContextModuleFactoryAfterResolve,
  NormalModuleFactoryResolveForScheme,
  NormalModuleFactoryCreateModule,
  NormalModuleFactoryAfterResolve,
  NormalModuleFactoryBeforeResolve,
  SucceedModule,
  StillValidModule,
  ExecuteModule,
  RuntimeModule,
}

impl From<String> for Hook {
  fn from(s: String) -> Self {
    match s.as_str() {
      "make" => Hook::Make,
      "finishMake" => Hook::FinishMake,
      "buildModule" => Hook::BuildModule,
      "thisCompilation" => Hook::ThisCompilation,
      "compilation" => Hook::Compilation,
      "processAssets" => Hook::ProcessAssets,
      "afterProcessAssets" => Hook::AfterProcessAssets,
      "emit" => Hook::Emit,
      "assetEmitted" => Hook::AssetEmitted,
      "shouldEmit" => Hook::ShouldEmit,
      "afterEmit" => Hook::AfterEmit,
      "optimizeChunkModules" => Hook::OptimizeChunkModules,
      "finishModules" => Hook::FinishModules,
      "optimizeModules" => Hook::OptimizeModules,
      "afterOptimizeModules" => Hook::AfterOptimizeModules,
      "optimizeTree" => Hook::OptimizeTree,
      "chunkAsset" => Hook::ChunkAsset,
      "contextModuleFactoryBeforeResolve" => Hook::ContextModuleFactoryBeforeResolve,
      "contextModuleFactoryAfterResolve" => Hook::ContextModuleFactoryAfterResolve,
      "normalModuleFactoryCreateModule" => Hook::NormalModuleFactoryCreateModule,
      "normalModuleFactoryResolveForScheme" => Hook::NormalModuleFactoryResolveForScheme,
      "normalModuleFactoryAfterResolve" => Hook::NormalModuleFactoryAfterResolve,
      "normalModuleFactoryBeforeResolve" => Hook::NormalModuleFactoryBeforeResolve,
      "succeedModule" => Hook::SucceedModule,
      "stillValidModule" => Hook::StillValidModule,
      "executeModule" => Hook::ExecuteModule,
      "runtimeModule" => Hook::RuntimeModule,
      hook_name => panic!("{hook_name} is an invalid hook name"),
    }
  }
}

#[derive(Default, Clone)]
pub struct DisabledHooks(Arc<RwLock<Vec<Hook>>>);

impl DisabledHooks {
  pub fn set_disabled_hooks(&self, hooks: Vec<String>) {
    let mut disabled_hooks = self.0.write().expect("failed to write lock");
    *disabled_hooks = hooks.into_iter().map(Into::into).collect::<Vec<Hook>>();
  }

  pub fn is_hook_disabled(&self, hook: &Hook) -> bool {
    self.0.read().expect("").contains(hook)
  }
}

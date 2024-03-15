use std::sync::RwLock;

/// rust support hooks
#[derive(PartialEq)]
pub enum Hook {
  FinishMake,
  BuildModule,
  ThisCompilation,
  AfterProcessAssets,
  Emit,
  AssetEmitted,
  AfterEmit,
  OptimizeChunkModules,
  BeforeCompile,
  AfterCompile,
  FinishModules,
  OptimizeModules,
  AfterOptimizeModules,
  OptimizeTree,
  /// webpack `compilation.hooks.chunkAsset`
  ChunkAsset,
  ContextModuleFactoryBeforeResolve,
  ContextModuleFactoryAfterResolve,
  NormalModuleFactoryResolveForScheme,
  NormalModuleFactoryCreateModule,
  AfterResolve,
  SucceedModule,
  StillValidModule,
  ExecuteModule,
  RuntimeModule,
}

impl From<String> for Hook {
  fn from(s: String) -> Self {
    match s.as_str() {
      "finishMake" => Hook::FinishMake,
      "buildModule" => Hook::BuildModule,
      "thisCompilation" => Hook::ThisCompilation,
      "afterProcessAssets" => Hook::AfterProcessAssets,
      "emit" => Hook::Emit,
      "assetEmitted" => Hook::AssetEmitted,
      "afterEmit" => Hook::AfterEmit,
      "optimizeChunkModules" => Hook::OptimizeChunkModules,
      "beforeCompile" => Hook::BeforeCompile,
      "afterCompile" => Hook::AfterCompile,
      "finishModules" => Hook::FinishModules,
      "optimizeModules" => Hook::OptimizeModules,
      "afterOptimizeModules" => Hook::AfterOptimizeModules,
      "optimizeTree" => Hook::OptimizeTree,
      "chunkAsset" => Hook::ChunkAsset,
      "contextModuleFactoryBeforeResolve" => Hook::ContextModuleFactoryBeforeResolve,
      "contextModuleFactoryAfterResolve" => Hook::ContextModuleFactoryAfterResolve,
      "normalModuleFactoryCreateModule" => Hook::NormalModuleFactoryCreateModule,
      "normalModuleFactoryResolveForScheme" => Hook::NormalModuleFactoryResolveForScheme,
      "afterResolve" => Hook::AfterResolve,
      "succeedModule" => Hook::SucceedModule,
      "stillValidModule" => Hook::StillValidModule,
      "executeModule" => Hook::ExecuteModule,
      "runtimeModule" => Hook::RuntimeModule,
      hook_name => panic!("{hook_name} is an invalid hook name"),
    }
  }
}

#[derive(Default)]
pub struct DisabledHooks(RwLock<Vec<Hook>>);

impl DisabledHooks {
  pub fn set_disabled_hooks(&self, hooks: Vec<String>) -> napi::Result<()> {
    let mut disabled_hooks = self.0.write().expect("failed to write lock");
    *disabled_hooks = hooks.into_iter().map(Into::into).collect::<Vec<Hook>>();
    Ok(())
  }

  pub fn is_hook_disabled(&self, hook: &Hook) -> bool {
    self.0.read().expect("").contains(hook)
  }
}

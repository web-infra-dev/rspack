use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_cacheable::with::Unsupported;
use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage,
  impl_runtime_module, rspack_sources::BoxSource, runtime_mode::RuntimeMode,
  runtime_module_get_generated_code,
};

type GenerateFn = Arc<dyn Fn() -> BoxFuture<'static, rspack_error::Result<String>> + Send + Sync>;

#[impl_runtime_module]
#[derive(Debug)]
pub struct RuntimeModuleFromJs {
  #[debug(skip)]
  #[cacheable(with=Unsupported)]
  pub generator: GenerateFn,
  pub full_hash: bool,
  pub dependent_hash: bool,
  pub isolate: bool,
  pub stage: RuntimeModuleStage,
}

#[async_trait::async_trait]
impl RuntimeModule for RuntimeModuleFromJs {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  async fn generate(&self, _: &RuntimeModuleGenerateContext<'_>) -> rspack_error::Result<String> {
    let res = (self.generator)().await?;
    Ok(res)
  }

  fn generated_source_for_bridge<'a>(
    &'a self,
    compilation: &'a Compilation,
  ) -> Option<BoxFuture<'a, rspack_error::Result<BoxSource>>> {
    Some(runtime_module_get_generated_code(
      self,
      &self.common,
      compilation,
    ))
  }

  fn full_hash(&self) -> bool {
    self.full_hash
  }

  fn dependent_hash(&self) -> bool {
    self.dependent_hash
  }

  fn should_isolate(&self, _runtime_mode: RuntimeMode) -> bool {
    self.isolate
  }

  fn stage(&self) -> RuntimeModuleStage {
    self.stage.clone()
  }
}

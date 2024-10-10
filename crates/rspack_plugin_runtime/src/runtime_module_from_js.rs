use std::sync::Arc;

use derivative::Derivative;
use rspack_cacheable::with::Unsupported;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule, RuntimeModuleStage,
};

type GenerateFn = Arc<dyn Fn() -> rspack_error::Result<String> + Send + Sync>;

#[impl_runtime_module]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct RuntimeModuleFromJs {
  pub name: String,
  #[derivative(Debug = "ignore")]
  #[cacheable(with=Unsupported)]
  pub generator: GenerateFn,
  pub cacheable: bool,
  pub isolate: bool,
  pub stage: RuntimeModuleStage,
}

impl RuntimeModule for RuntimeModuleFromJs {
  fn name(&self) -> Identifier {
    Identifier::from(format!("webpack/runtime/{}", self.name))
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    let res = (self.generator)()?;
    Ok(RawSource::from(res).boxed())
  }

  fn cacheable(&self) -> bool {
    self.cacheable
  }

  fn should_isolate(&self) -> bool {
    self.isolate
  }

  fn stage(&self) -> RuntimeModuleStage {
    self.stage.clone()
  }
}

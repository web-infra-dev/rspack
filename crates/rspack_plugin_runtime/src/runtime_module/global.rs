use async_trait::async_trait;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GlobalRuntimeModule {
  id: Identifier,
}

impl Default for GlobalRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/global"))
  }
}

#[async_trait]
impl RuntimeModule for GlobalRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawStringSource::from_static(include_str!("runtime/global.js")).boxed())
  }
}

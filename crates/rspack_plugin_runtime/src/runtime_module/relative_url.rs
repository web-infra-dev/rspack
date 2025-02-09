use async_trait::async_trait;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RelativeUrlRuntimeModule {
  id: Identifier,
}

impl Default for RelativeUrlRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/relative_url"))
  }
}

#[async_trait]
impl RuntimeModule for RelativeUrlRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawStringSource::from_static(include_str!("runtime/relative_url.js")).boxed())
  }
}

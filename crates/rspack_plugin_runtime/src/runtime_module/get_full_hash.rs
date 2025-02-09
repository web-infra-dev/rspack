use async_trait::async_trait;
use cow_utils::CowUtils;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetFullHashRuntimeModule {
  id: Identifier,
}

impl Default for GetFullHashRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/get_full_hash"))
  }
}

#[async_trait]
impl RuntimeModule for GetFullHashRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawStringSource::from(
        include_str!("runtime/get_full_hash.js")
          .cow_replace("$HASH$", compilation.get_hash().unwrap_or("XXXX"))
          .into_owned(),
      )
      .boxed(),
    )
  }

  fn full_hash(&self) -> bool {
    true
  }
}

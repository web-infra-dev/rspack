use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct GetFullHashRuntimeModule {}

impl RuntimeModule for GetFullHashRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/get_full_hash".to_string()
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    RawSource::from(
      include_str!("runtime/get_full_hash.js").replace("$HASH$", compilation.hash.as_str()),
    )
    .boxed()
  }
}

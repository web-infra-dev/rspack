use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
pub struct GetFullHashRuntimeModule {}

impl RuntimeModule for GetFullHashRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/get_full_hash".to_owned()
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    RawSource::from(
      include_str!("runtime/get_full_hash.js").replace("$HASH$", compilation.hash.as_str()),
    )
    .boxed()
  }
}

impl_runtime_module!(GetFullHashRuntimeModule);

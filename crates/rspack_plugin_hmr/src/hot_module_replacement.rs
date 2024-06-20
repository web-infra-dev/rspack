use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::test::{HOT_TEST_DEFINE_GLOBAL, HOT_TEST_STATUS_CHANGE};

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementRuntimeModule {
  id: Identifier,
}

impl Default for HotModuleReplacementRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/hot_module_replacement"))
  }
}

impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(
        include_str!("runtime/hot_module_replacement.js")
          .replace("$HOT_TEST_GLOBAL$", &HOT_TEST_DEFINE_GLOBAL)
          .replace("$HOT_TEST_STATUS$", &HOT_TEST_STATUS_CHANGE),
      )
      .boxed(),
    )
  }
}

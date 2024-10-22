use cow_utils::CowUtils;
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
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
    let generated_code = include_str!("runtime/hot_module_replacement.js")
      .cow_replace("$HOT_TEST_GLOBAL$", &HOT_TEST_DEFINE_GLOBAL)
      .cow_replace("$HOT_TEST_STATUS$", &HOT_TEST_STATUS_CHANGE)
      .to_string();

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}

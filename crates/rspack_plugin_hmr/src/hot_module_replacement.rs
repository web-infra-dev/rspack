use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

pub fn is_hot_test() -> bool {
  let is_hot_test = std::env::var("RSPACK_HOT_TEST").ok().unwrap_or_default();
  is_hot_test == "true"
}

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct HotModuleReplacementRuntimeModule {
  id: Identifier,
}

impl Default for HotModuleReplacementRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/hot_module_replacement"),
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(if is_hot_test() {
        include_str!("runtime/hot_module_replacement_test.js")
      } else {
        include_str!("runtime/hot_module_replacement.js")
      })
      .boxed(),
    )
  }
}

use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule, SourceMapOption,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct HarmonyModuleDecoratorRuntimeModule {
  id: Identifier,
}

impl Default for HarmonyModuleDecoratorRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/harmony_module_decorator"),
      source_map_option: SourceMapOption::None,
    }
  }
}

impl RuntimeModule for HarmonyModuleDecoratorRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/harmony_module_decorator.js")).boxed()
  }
}

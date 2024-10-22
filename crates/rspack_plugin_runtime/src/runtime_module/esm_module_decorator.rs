use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ESMModuleDecoratorRuntimeModule {
  id: Identifier,
}

impl Default for ESMModuleDecoratorRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/esm_module_decorator"))
  }
}

impl RuntimeModule for ESMModuleDecoratorRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let generated_code = include_str!("runtime/esm_module_decorator.js").to_string();

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}

use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl Default for CreateFakeNamespaceObjectRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from(
      "webpack/runtime/create_fake_namespace_object",
    ))
  }
}

impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let generated_code = include_str!("runtime/create_fake_namespace_object.js").to_string();

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}

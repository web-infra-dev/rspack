use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPrefetchPreloadFunctionRuntimeModule {
  id: Identifier,
  runtime_function: RuntimeGlobals,
  runtime_handlers: RuntimeGlobals,
}

impl ChunkPrefetchPreloadFunctionRuntimeModule {
  pub fn new(
    child_type: &str,
    runtime_function: RuntimeGlobals,
    runtime_handlers: RuntimeGlobals,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "webpack/runtime/chunk_prefetch_function/{}",
        child_type
      )),
      runtime_function,
      runtime_handlers,
    )
  }
}

impl RuntimeModule for ChunkPrefetchPreloadFunctionRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/chunk_prefetch_preload_function.ejs").to_string(),
    )]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "RUNTIME_HANDLERS":  &self.runtime_handlers.to_string(),
        "RUNTIME_FUNCTION": &self.runtime_function.to_string(),
      })),
    )?;

    Ok(RawStringSource::from(source).boxed())
  }
}

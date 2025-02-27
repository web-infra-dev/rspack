use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GlobalRuntimeModule {
  id: Identifier,
}

impl Default for GlobalRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/global"))
  }
}

impl RuntimeModule for GlobalRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/global.ejs").to_string(),
    )]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(RawStringSource::from(source).boxed())
  }
}

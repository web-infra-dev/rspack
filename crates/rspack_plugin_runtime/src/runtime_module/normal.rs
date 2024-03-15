use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct NormalRuntimeModule {
  pub identifier: Identifier,
  pub sources: &'static str,
}

impl NormalRuntimeModule {
  pub fn new(identifier: RuntimeGlobals, sources: &'static str) -> Self {
    Self {
      identifier: Identifier::from(identifier.name()),
      sources,
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for NormalRuntimeModule {
  fn name(&self) -> Identifier {
    self.identifier
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(self.sources).boxed())
  }
}

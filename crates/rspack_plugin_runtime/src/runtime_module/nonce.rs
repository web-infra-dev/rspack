use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct NonceRuntimeModule {
  id: Identifier,
}

impl Default for NonceRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/nonce"),
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for NonceRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(format!("{} = undefined;", RuntimeGlobals::SCRIPT_NONCE)).boxed())
  }
}

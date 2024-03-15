use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct GetFullHashRuntimeModule {
  id: Identifier,
}

impl Default for GetFullHashRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/get_full_hash"),
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for GetFullHashRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(
        include_str!("runtime/get_full_hash.js")
          .replace("$HASH$", compilation.get_hash().unwrap_or("XXXX")),
      )
      .boxed(),
    )
  }

  fn cacheable(&self) -> bool {
    false
  }
}

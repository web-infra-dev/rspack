use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct RspackExposeGlobalRuntimeModule {
  id: Identifier,
  global: String,
}

impl RspackExposeGlobalRuntimeModule {
  pub fn new(global: String) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/rspack_expose_global"),
      global,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for RspackExposeGlobalRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let unique_name = if compilation.options.output.unique_name.is_empty() {
      format!("{}()", RuntimeGlobals::GET_FULL_HASH)
    } else {
      format!("\"{}\"", compilation.options.output.unique_name)
    };
    Ok(
      RawSource::from(
        include_str!("runtime/expose_global.js")
          .replace("$GLOBAL$", &self.global)
          .replace("$UNIQUE_NAME$", &unique_name),
      )
      .boxed(),
    )
  }
}

use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct FederationRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for FederationRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("federation/runtime"),
      chunk: None,
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for FederationRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Normal
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(federation_runtime_template()).boxed())
  }
}

fn federation_runtime_template() -> String {
  let federation_global = format!("{}.federation", RuntimeGlobals::REQUIRE);
  format!(
    r#"
if(!{federation_global}){{
  {federation_global} = {{}};
}}
"#,
    federation_global = federation_global,
  )
}

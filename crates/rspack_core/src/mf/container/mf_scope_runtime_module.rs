use rspack_identifier::Identifier;
use rspack_sources::{BoxSource, RawSource, SourceExt};

use crate::{impl_runtime_module, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule};

#[derive(Debug, Eq)]
pub struct MFScopeRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for MFScopeRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/mf_scope"),
      chunk: None,
    }
  }
}

impl RuntimeModule for MFScopeRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> BoxSource {
    RawSource::from(format!(r#"{}.MF = {{}};"#, RuntimeGlobals::REQUIRE)).boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(MFScopeRuntimeModule);

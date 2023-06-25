use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct BaseUriRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}
impl Default for BaseUriRuntimeModule {
  fn default() -> Self {
    BaseUriRuntimeModule {
      id: Identifier::from("webpack/runtime/base_uri"),
      chunk: None,
    }
  }
}

impl RuntimeModule for BaseUriRuntimeModule {
  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let base_uri = self
      .chunk
      .and_then(|ukey| compilation.chunk_by_ukey.get(&ukey))
      .and_then(|chunk| chunk.get_entry_options(&compilation.chunk_group_by_ukey))
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| "undefined".to_string());
    RawSource::from(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)).boxed()
  }

  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
impl_runtime_module!(BaseUriRuntimeModule);

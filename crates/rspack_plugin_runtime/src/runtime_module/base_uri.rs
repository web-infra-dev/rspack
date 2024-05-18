use rspack_core::{
  get_chunk_from_ukey, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
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
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for BaseUriRuntimeModule {
  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let base_uri = self
      .chunk
      .and_then(|ukey| get_chunk_from_ukey(&ukey, &compilation.chunk_by_ukey))
      .and_then(|chunk| chunk.get_entry_options(&compilation.chunk_group_by_ukey))
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| "undefined".to_string());
    Ok(RawSource::from(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)).boxed())
  }

  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

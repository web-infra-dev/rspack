use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  ChunkUkey, Compilation, FilenameTemplate, PathData, RuntimeGlobals, RuntimeModule,
};
use rspack_util::infallible::ResultInfallibleExt;

// TODO workaround for get_chunk_update_filename
#[impl_runtime_module]
#[derive(Debug)]
pub struct GetChunkUpdateFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for GetChunkUpdateFilenameRuntimeModule {
  fn default() -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/get_chunk_update_filename"),
      None,
    )
  }
}

impl RuntimeModule for GetChunkUpdateFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename = compilation
        .get_path(
          &FilenameTemplate::from(compilation.options.output.hot_update_chunk_filename.clone()),
          PathData::default()
            .chunk(chunk)
            .hash(format!("' + {}() + '", RuntimeGlobals::GET_FULL_HASH).as_str())
            .id("' + chunkId + '")
            .runtime(&chunk.runtime),
        )
        .always_ok();
      let generated_code = format!(
        "{} = function (chunkId) {{
            return '{}';
         }};
        ",
        RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME,
        filename
      );

      let source = if self.source_map_kind.enabled() {
        OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
      } else {
        RawSource::from(generated_code).boxed()
      };
      Ok(source)
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }
}

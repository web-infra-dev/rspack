use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, FilenameTemplate, PathData, RuntimeGlobals, RuntimeModule, SourceType,
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

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_chunk_update_filename.ejs").to_string(),
    )]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename = compilation
        .get_path(
          &FilenameTemplate::from(compilation.options.output.hot_update_chunk_filename.clone()),
          PathData::default()
            .chunk_hash_optional(chunk.rendered_hash(
              &compilation.chunk_hashes_artifact,
              compilation.options.output.hash_digest_length,
            ))
            .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids_artifact))
            .content_hash_optional(chunk.rendered_content_hash_by_source_type(
              &compilation.chunk_hashes_artifact,
              &SourceType::JavaScript,
              compilation.options.output.hash_digest_length,
            ))
            .hash(format!("' + {}() + '", RuntimeGlobals::GET_FULL_HASH).as_str())
            .id("' + chunkId + '")
            .runtime(chunk.runtime().as_str()),
        )
        .always_ok();

      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "_filename": format!("'{}'", filename),
        })),
      )?;

      Ok(RawStringSource::from(source).boxed())
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

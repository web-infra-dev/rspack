use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, PathData, RuntimeGlobals, RuntimeModule, RuntimeTemplate, SourceType,
  impl_runtime_module,
};

// TODO workaround for get_chunk_update_filename
#[impl_runtime_module]
#[derive(Debug)]
pub struct GetChunkUpdateFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl GetChunkUpdateFilenameRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}get_chunk_update_filename",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }
}

#[async_trait::async_trait]
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

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename = compilation
        .get_path(
          &compilation.options.output.hot_update_chunk_filename,
          PathData::default()
            .chunk_hash_optional(chunk.rendered_hash(
              &compilation.chunk_hashes_artifact,
              compilation.options.output.hash_digest_length,
            ))
            .chunk_name_optional(chunk.name_for_filename_template())
            .content_hash_optional(chunk.rendered_content_hash_by_source_type(
              &compilation.chunk_hashes_artifact,
              &SourceType::JavaScript,
              compilation.options.output.hash_digest_length,
            ))
            .hash(
              format!(
                "' + {}() + '",
                compilation
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::GET_FULL_HASH)
              )
              .as_str(),
            )
            .id("' + chunkId + '")
            .runtime(chunk.runtime().as_str()),
        )
        .await?;

      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "_filename": format!("'{}'", filename),
        })),
      )?;

      Ok(source)
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

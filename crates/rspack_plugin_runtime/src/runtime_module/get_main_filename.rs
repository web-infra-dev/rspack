use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, Filename, PathData, RuntimeGlobals, RuntimeModule, SourceType,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetMainFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  global: RuntimeGlobals,
  filename: Filename,
}

impl GetMainFilenameRuntimeModule {
  pub fn new(content_type: &'static str, global: RuntimeGlobals, filename: Filename) -> Self {
    Self::with_default(
      Identifier::from(format!("webpack/runtime/get_main_filename/{content_type}")),
      None,
      global,
      filename,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetMainFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename = compilation
        .get_path(
          &self.filename,
          PathData::default()
            .chunk_id_optional(
              chunk
                .id(&compilation.chunk_ids_artifact)
                .map(|id| id.as_str()),
            )
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
            .hash(format!("\" + {}() + \"", RuntimeGlobals::GET_FULL_HASH).as_str())
            .runtime(chunk.runtime().as_str()),
        )
        .await?;

      Ok(format!(
        "{} = function () {{
            return \"{}\";
         }};
        ",
        self.global, filename
      ))
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

use rspack_core::{
  Compilation, Filename, PathData, RuntimeGlobals, RuntimeModule, RuntimeTemplate, SourceType,
  has_hash_placeholder, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetMainFilenameRuntimeModule {
  global: RuntimeGlobals,
  filename: Filename,
}

impl GetMainFilenameRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    content_type: &'static str,
    global: RuntimeGlobals,
    filename: Filename,
  ) -> Self {
    Self::with_name(
      runtime_template,
      &format!("get_main_filename/{content_type}"),
      global,
      filename,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetMainFilenameRuntimeModule {
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let filename = compilation
        .get_path(
          &self.filename,
          PathData::default()
            .chunk_id_optional(chunk.id().map(|id| id.as_str()))
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
                "\" + {}() + \"",
                compilation
                  .runtime_template
                  .render_runtime_globals(&RuntimeGlobals::GET_FULL_HASH)
              )
              .as_str(),
            )
            .runtime(chunk.runtime().as_str()),
        )
        .await?;

      Ok(format!(
        "{} = function () {{
            return \"{}\";
         }};
        ",
        compilation
          .runtime_template
          .render_runtime_globals(&self.global),
        filename,
      ))
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn additional_runtime_requirements(&self, compilation: &Compilation) -> RuntimeGlobals {
    if has_hash_placeholder(compilation.options.output.hot_update_main_filename.as_str()) {
      RuntimeGlobals::GET_FULL_HASH
    } else {
      RuntimeGlobals::default()
    }
  }
}

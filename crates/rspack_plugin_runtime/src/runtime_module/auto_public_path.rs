use rspack_collections::Identifier;
use rspack_core::{
  get_js_chunk_filename_template, get_undo_path, impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, OutputOptions, PathData, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, RuntimeTemplate, SourceType,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AutoPublicPathRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for AutoPublicPathRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/auto_public_path"), None)
  }
}

impl RuntimeModule for AutoPublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/auto_public_path.ejs").to_string(),
    )]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = self.chunk.expect("The chunk should be attached");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
    let filename = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let filename = compilation.get_path(
      &filename,
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
        )),
    )?;
    Ok(
      RawStringSource::from(auto_public_path_template(
        &compilation.runtime_template,
        &self.id,
        &filename,
        &compilation.options.output,
      )?)
      .boxed(),
    )
  }
}

fn auto_public_path_template(
  runtime_template: &RuntimeTemplate,
  id: &str,
  filename: &str,
  output: &OutputOptions,
) -> rspack_error::Result<String> {
  let output_path = output.path.as_str().to_string();
  let undo_path = get_undo_path(filename, output_path, false);
  let assign = if undo_path.is_empty() {
    format!("{} = scriptUrl", RuntimeGlobals::PUBLIC_PATH)
  } else {
    format!(
      "{} = scriptUrl + '{undo_path}'",
      RuntimeGlobals::PUBLIC_PATH
    )
  };
  let import_meta_name = output.import_meta_name.clone();

  runtime_template.render(
    id,
    Some(serde_json::json!({
      "script_type": output.script_type,
      "import_meta_name": import_meta_name,
      "assign": assign
    })),
  )
}

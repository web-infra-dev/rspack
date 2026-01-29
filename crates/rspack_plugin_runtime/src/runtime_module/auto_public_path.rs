use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, OutputOptions, PathData, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, RuntimeTemplate, SourceType, get_js_chunk_filename_template, get_undo_path,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AutoPublicPathRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl AutoPublicPathRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}auto_public_path",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }
}

#[async_trait::async_trait]
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

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk = self.chunk.expect("The chunk should be attached");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
    let filename = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let filename = compilation
      .get_path(
        &filename,
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
          )),
      )
      .await?;
    auto_public_path_template(
      &compilation.runtime_template,
      &self.id,
      &filename,
      &compilation.options.output,
    )
  }

  fn additional_runtime_requirements(
    &self,
    _compilation: &Compilation,
    _runtime_requirements: &RuntimeGlobals,
  ) -> RuntimeGlobals {
    RuntimeGlobals::GLOBAL
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
  let import_meta_name = output.import_meta_name.clone();

  runtime_template.render(
    id,
    Some(serde_json::json!({
      "_script_type": output.script_type,
      "_import_meta_name": import_meta_name,
      "_undo_path": undo_path
    })),
  )
}

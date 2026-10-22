use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeTemplate, impl_runtime_module,
};

/// Front-code sorted block identifiers. Lengths use JavaScript's UTF-16 units.
pub fn encode_block_chunks<'a, T>(
  blocks: impl IntoIterator<Item = (&'a str, T)>,
) -> Vec<(usize, &'a str, T)> {
  let mut previous = "";
  blocks
    .into_iter()
    .map(|(id, chunks)| {
      let mut bytes = 0;
      let mut units = 0;
      for ((offset, current), old) in id.char_indices().zip(previous.chars()) {
        if current != old {
          break;
        }
        bytes = offset + current.len_utf8();
        units += current.len_utf16();
      }
      previous = id;
      (units, &id[bytes..], chunks)
    })
    .collect()
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct BlockChunkMapRuntimeModule {}

impl BlockChunkMapRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for BlockChunkMapRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Basic
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/block_chunk_map.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let graph = &compilation.build_chunk_graph_artifact;
    let chunk = graph
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("should have chunk"));
    let blocks = graph
      .chunk_graph
      .get_block_chunks(compilation, Some(chunk.runtime()));
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_blocks": rspack_util::json_stringify(&encode_block_chunks(blocks.iter().map(|(id, chunks)| (id.as_str(), chunks)))),
      })),
    )
  }

  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE,
      ..Default::default()
    }
  }
}

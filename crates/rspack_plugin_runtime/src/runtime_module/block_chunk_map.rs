use std::{
  collections::BTreeMap,
  sync::{Arc, OnceLock},
};

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeModuleUpdateData, RuntimeTemplate,
  chunk_graph_chunk::ChunkId, impl_runtime_module,
};

#[derive(Debug)]
pub struct BlockChunkMapData(pub BTreeMap<String, Vec<ChunkId>>);

impl RuntimeModuleUpdateData for BlockChunkMapData {}

#[impl_runtime_module]
#[derive(Debug)]
pub struct BlockChunkMapRuntimeModule {
  #[cacheable(with=rspack_cacheable::with::Skip)]
  data: OnceLock<Arc<BlockChunkMapData>>,
}

impl BlockChunkMapRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template, Default::default())
  }

  fn data(&self, compilation: &Compilation) -> &Arc<BlockChunkMapData> {
    self.data.get_or_init(|| {
      let graph = &compilation.build_chunk_graph_artifact;
      let chunk = graph
        .chunk_by_ukey
        .expect_get(&self.chunk().expect("should have chunk"));
      Arc::new(BlockChunkMapData(
        graph
          .chunk_graph
          .get_block_chunks(compilation, chunk.runtime()),
      ))
    })
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
    let blocks = &self.data(context.compilation).0;
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_full": true,
        "_blocks": rspack_util::json_stringify(blocks),
      })),
    )
  }

  fn get_update_data(&self, compilation: &Compilation) -> Option<Arc<dyn RuntimeModuleUpdateData>> {
    let data = Arc::clone(self.data(compilation));
    Some(data)
  }

  async fn generate_update(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
    previous: &dyn RuntimeModuleUpdateData,
  ) -> rspack_error::Result<Option<String>> {
    let Some(previous) = previous.as_any().downcast_ref::<BlockChunkMapData>() else {
      return Ok(None);
    };
    let blocks = &self.data(context.compilation).0;
    let changes = blocks
      .iter()
      .filter(|(id, chunks)| previous.0.get(*id) != Some(*chunks))
      .map(|(id, chunks)| (id.as_str(), Some(chunks)))
      .chain(
        previous
          .0
          .keys()
          .filter(|id| !blocks.contains_key(*id))
          .map(|id| (id.as_str(), None)),
      )
      .collect::<BTreeMap<_, _>>();
    if changes.is_empty() {
      return Ok(Some(String::new()));
    }
    context
      .runtime_template
      .render(
        self.id(),
        Some(serde_json::json!({
          "_full": false,
          "_blocks": rspack_util::json_stringify(&changes),
        })),
      )
      .map(Some)
  }

  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE,
      ..Default::default()
    }
  }
}

use std::{
  collections::BTreeMap,
  sync::{Arc, OnceLock},
};

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeModuleUpdateData, RuntimeTemplate,
  chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct ChunkGroupMapData(pub BTreeMap<ChunkId, Vec<ChunkId>>);

impl RuntimeModuleUpdateData for ChunkGroupMapData {}

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkGroupMapRuntimeModule {
  #[cacheable(with=rspack_cacheable::with::Skip)]
  data: OnceLock<Arc<ChunkGroupMapData>>,
}

impl ChunkGroupMapRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template, Default::default())
  }

  fn data(&self, compilation: &Compilation) -> &Arc<ChunkGroupMapData> {
    self.data.get_or_init(|| {
      let graph = &compilation.build_chunk_graph_artifact;
      let chunk = graph
        .chunk_by_ukey
        .expect_get(&self.chunk().expect("should have chunk"));
      let mut groups = BTreeMap::new();
      let mut queue = chunk.groups().iter().copied().collect::<Vec<_>>();
      let mut visited = FxHashSet::default();
      while let Some(key) = queue.pop() {
        if !visited.insert(key) {
          continue;
        }
        let group = graph.chunk_group_by_ukey.expect_get(&key);
        // Follow this runtime's groups, not every group of a shared chunk.
        queue.extend(group.children.iter().copied());
        let Some(last) = group.get_hmr_loading_chunk(compilation) else {
          continue;
        };
        let siblings = group
          .get_loading_chunks(compilation)
          .filter(|chunk| chunk.ukey() != last.ukey())
          .map(|chunk| chunk.expect_id().clone())
          .collect::<Vec<_>>();
        if !siblings.is_empty() {
          groups.insert(last.expect_id().clone(), siblings);
        }
      }
      Arc::new(ChunkGroupMapData(groups))
    })
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkGroupMapRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Basic
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/chunk_group_map.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let groups = &self.data(context.compilation).0;
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_full": true,
        "_groups": rspack_util::json_stringify(groups),
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
    let Some(previous) = previous.as_any().downcast_ref::<ChunkGroupMapData>() else {
      return Ok(None);
    };
    let groups = &self.data(context.compilation).0;
    let changes = groups
      .iter()
      .filter(|(id, chunks)| previous.0.get(*id) != Some(*chunks))
      .map(|(id, chunks)| (id, Some(chunks)))
      .chain(
        previous
          .0
          .keys()
          .filter(|id| !groups.contains_key(*id))
          .map(|id| (id, None)),
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
          "_groups": rspack_util::json_stringify(&changes),
        })),
      )
      .map(Some)
  }

  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE | RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      ..Default::default()
    }
  }
}

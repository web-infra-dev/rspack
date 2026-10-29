use std::{
  collections::{BTreeMap, BTreeSet},
  sync::LazyLock,
};

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeTemplate, impl_runtime_module,
};
use rspack_plugin_runtime::extract_runtime_module_variables_from_ejs;
const TEMPLATE: &str = include_str!("runtime/chunk_groups.ejs");
static VARIABLES: LazyLock<Vec<&'static str>> =
  LazyLock::new(|| extract_runtime_module_variables_from_ejs(&[TEMPLATE]));

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementChunkGroupsRuntimeModule {}
impl HotModuleReplacementChunkGroupsRuntimeModule {
  pub fn new(template: &RuntimeTemplate) -> Self {
    Self::with_default(template)
  }
}
#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementChunkGroupsRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &VARIABLES
  }
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }
  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), TEMPLATE.to_string())]
  }
  fn runtime_requirements(&self, _: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      ..Default::default()
    }
  }
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let graph = &context.compilation.build_chunk_graph_artifact;
    let root = graph
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("runtime chunk"));
    let mut map = BTreeMap::<_, BTreeSet<_>>::new();
    for key in root.get_all_async_chunks(&graph.chunk_group_by_ukey) {
      for group in graph.chunk_by_ukey.expect_get(&key).groups() {
        let group = graph.chunk_group_by_ukey.expect_get(group);
        if group.is_initial() {
          continue;
        }
        let chunks = group
          .chunks
          .iter()
          .map(|key| graph.chunk_by_ukey.expect_get(key))
          .filter(|chunk| !chunk.has_runtime(&graph.chunk_group_by_ukey))
          .filter_map(|chunk| chunk.id())
          .collect::<Vec<_>>();
        if chunks.len() > 1 {
          map
            .entry(chunks[chunks.len() - 1])
            .or_default()
            .extend(chunks[..chunks.len() - 1].iter().copied());
        }
      }
    }
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({"_groups": serde_json::to_string(&map).expect("chunk groups")})),
    )
  }
}

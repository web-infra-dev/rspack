use std::{collections::BTreeMap, sync::Arc};

use rspack_core::{
  Chunk, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleUpdateData, RuntimeTemplate, SourceType,
  chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rspack_plugin_javascript::runtime::get_hmr_chunk_version;

pub(crate) fn is_versioned(compilation: &Compilation, chunk: &Chunk) -> bool {
  let graph = &compilation.build_chunk_graph_artifact;
  !chunk.has_runtime(&graph.chunk_group_by_ukey)
    // A shared chunk can be initial in one runtime and async in another.
    && chunk.groups().iter().any(|group| !graph.chunk_group_by_ukey.expect_get(group).is_initial())
    && !graph.chunk_graph.get_chunk_modules_identifier_by_source_type(
      &chunk.ukey(), SourceType::JavaScript, compilation.get_module_graph()
    ).is_empty()
}

#[derive(Debug)]
struct ChunkVersions(BTreeMap<ChunkId, String>);

impl RuntimeModuleUpdateData for ChunkVersions {}

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementChunkVersionsRuntimeModule {}

impl HotModuleReplacementChunkVersionsRuntimeModule {
  pub fn new(template: &RuntimeTemplate) -> Self {
    Self::with_default(template)
  }

  fn versions(&self, compilation: &Compilation) -> BTreeMap<ChunkId, String> {
    let graph = &compilation.build_chunk_graph_artifact;
    let root = graph
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("should have runtime chunk"));
    root
      .get_all_referenced_chunks(&graph.chunk_group_by_ukey)
      .iter()
      .map(|key| graph.chunk_by_ukey.expect_get(key))
      .filter(|chunk| is_versioned(compilation, chunk))
      .map(|chunk| {
        (
          chunk.expect_id().clone(),
          get_hmr_chunk_version(compilation, chunk),
        )
      })
      .collect()
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementChunkVersionsRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(&self, _: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      // Updates mutate the existing map; the renderer owns its shared binding.
      dependencies: RuntimeGlobals::HMR_CHUNK_VERSIONS,
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let map = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::HMR_CHUNK_VERSIONS);
    let entries = self
      .versions(context.compilation)
      .into_iter()
      .collect::<Vec<_>>();
    Ok(format!(
      "{map} = Object.create(null);\n{}.forEach(function(entry) {{ {map}[entry[0]] = entry[1]; }});",
      rspack_util::json_stringify(&entries)
    ))
  }

  fn get_update_data(&self, compilation: &Compilation) -> Option<Arc<dyn RuntimeModuleUpdateData>> {
    Some(Arc::new(ChunkVersions(self.versions(compilation))))
  }

  async fn generate_update(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
    previous: &dyn RuntimeModuleUpdateData,
  ) -> rspack_error::Result<Option<String>> {
    let Some(previous) = previous.as_any().downcast_ref::<ChunkVersions>() else {
      return Ok(None);
    };
    let versions = self.versions(context.compilation);
    let map = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::HMR_CHUNK_VERSIONS);
    let mut source = String::new();
    for (id, version) in &versions {
      if previous.0.get(id) != Some(version) {
        source.push_str(&format!(
          "{map}[{}] = {};\n",
          rspack_util::json_stringify(id),
          rspack_util::json_stringify_str(version),
        ));
      }
    }
    for id in previous.0.keys().filter(|id| !versions.contains_key(*id)) {
      source.push_str(&format!(
        "delete {map}[{}];\n",
        rspack_util::json_stringify(id)
      ));
    }
    Ok(Some(source))
  }
}

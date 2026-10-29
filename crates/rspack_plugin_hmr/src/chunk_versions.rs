use std::{collections::BTreeMap, hash::Hash};

use rspack_core::{
  Chunk, ChunkGraph, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeTemplate, SourceType, impl_runtime_module,
};
use rspack_hash::RspackHasher;

pub(crate) fn is_versioned(compilation: &Compilation, chunk: &Chunk) -> bool {
  let graph = &compilation.build_chunk_graph_artifact;
  !chunk.has_runtime(&graph.chunk_group_by_ukey)
    && !chunk.can_be_initial(&graph.chunk_group_by_ukey)
    && chunk.id().is_some()
    && !graph
      .chunk_graph
      .get_chunk_modules_identifier_by_source_type(
        &chunk.ukey(),
        SourceType::JavaScript,
        compilation.get_module_graph(),
      )
      .is_empty()
}

fn version(compilation: &Compilation, chunk: &Chunk) -> String {
  let mut modules = compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_identifier_by_source_type(
      &chunk.ukey(),
      SourceType::JavaScript,
      compilation.get_module_graph(),
    );
  modules.sort_unstable();
  let output = &compilation.options.output;
  let mut hasher = RspackHasher::from(output);
  for module in modules {
    ChunkGraph::get_module_id(&compilation.module_ids_artifact, module).hash(&mut hasher);
    ChunkGraph::get_module_hash(compilation, module, chunk.runtime()).hash(&mut hasher);
  }
  hasher
    .digest(&output.hash_digest)
    .rendered(output.hash_digest_length)
    .to_string()
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementChunkVersionsRuntimeModule {}
impl HotModuleReplacementChunkVersionsRuntimeModule {
  pub fn new(template: &RuntimeTemplate) -> Self {
    Self::with_default(template)
  }
}
#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementChunkVersionsRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }
  fn runtime_requirements(&self, _: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE,
      ..Default::default()
    }
  }
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let graph = &compilation.build_chunk_graph_artifact;
    let root = graph
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("runtime chunk"));
    let versions = root
      .get_all_async_chunks(&graph.chunk_group_by_ukey)
      .iter()
      .map(|key| graph.chunk_by_ukey.expect_get(key))
      .filter(|chunk| is_versioned(compilation, chunk))
      .map(|chunk| (chunk.expect_id(), version(compilation, chunk)))
      .collect::<BTreeMap<_, _>>();
    Ok(format!(
      "{}.hmrV = {};",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE),
      serde_json::to_string(&versions).expect("chunk versions")
    ))
  }
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementLoadedChunkVersionRuntimeModule {}
impl HotModuleReplacementLoadedChunkVersionRuntimeModule {
  pub fn new(template: &RuntimeTemplate, chunk_id: &str) -> Self {
    Self::with_name(
      template,
      &format!("hot_module_replacement_loaded_chunk_version|{chunk_id}"),
    )
  }
}
#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementLoadedChunkVersionRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }
  fn runtime_requirements(&self, _: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE,
      ..Default::default()
    }
  }
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("async chunk"));
    Ok(format!(
      "{}.hmrL[{}] = {};",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE),
      serde_json::to_string(chunk.expect_id()).expect("chunk id"),
      serde_json::to_string(&version(compilation, chunk)).expect("chunk version")
    ))
  }
}

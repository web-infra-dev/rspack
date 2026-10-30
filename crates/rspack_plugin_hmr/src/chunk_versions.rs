// Ported from webpack's HotModuleReplacementChunkVersionsRuntimeModule and
// HotModuleReplacementLoadedChunkVersionRuntimeModule (MIT licensed).
use std::{collections::BTreeMap, hash::Hash};

use rspack_core::{
  Chunk, ChunkGraph, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeTemplate, SourceType, impl_runtime_module,
  runtime_mode::RuntimeMode,
};
use rspack_hash::RspackHasher;

pub(crate) fn is_versioned(compilation: &Compilation, chunk: &Chunk) -> bool {
  let graph = &compilation.build_chunk_graph_artifact;
  !chunk.has_runtime(&graph.chunk_group_by_ukey)
    && chunk.id().is_some()
    // A chunk can be initial in one runtime and async in another.
    && chunk
      .groups()
      .iter()
      .any(|group| !graph.chunk_group_by_ukey.expect_get(group).is_initial())
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
      .expect_get(&self.chunk().expect("should have runtime chunk"));
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
      rspack_util::json_stringify(&versions)
    ))
  }
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementLoadedChunkVersionRuntimeModule {}

impl HotModuleReplacementLoadedChunkVersionRuntimeModule {
  pub fn new(template: &RuntimeTemplate, chunk_id: &str) -> Self {
    // Each chunk needs its own marker even when several chunks share a runtime.
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

  fn should_isolate(&self, _: RuntimeMode) -> bool {
    false
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
      .expect_get(&self.chunk().expect("should have async chunk"));
    Ok(format!(
      "{}.hmrL[{}] = {};",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE),
      rspack_util::json_stringify(chunk.expect_id()),
      rspack_util::json_stringify(&version(compilation, chunk))
    ))
  }
}

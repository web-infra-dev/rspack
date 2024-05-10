use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Chunk, ChunkUkey, Compilation, ModuleIdentifier, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap;

use super::consume_shared_plugin::ConsumeVersion;
use crate::utils::json_stringify;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct ConsumeSharedRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  enhanced: bool,
}

impl ConsumeSharedRuntimeModule {
  pub fn new(enhanced: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/consumes_loading"),
      chunk: None,
      enhanced,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for ConsumeSharedRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let module_graph = compilation.get_module_graph();
    let mut chunk_to_module_mapping = FxHashMap::default();
    let mut module_id_to_consume_data_mapping = FxHashMap::default();
    let mut initial_consumes = Vec::new();
    let mut add_module = |module: ModuleIdentifier, chunk: &Chunk, ids: &mut Vec<String>| {
      let id = compilation
        .chunk_graph
        .get_module_id(module)
        .clone()
        .expect("should have moduleId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
      ids.push(id.clone());
      let code_gen = compilation
        .code_generation_results
        .get(&module, Some(&chunk.runtime));
      if let Some(data) = code_gen.data.get::<CodeGenerationDataConsumeShared>() {
        module_id_to_consume_data_mapping.insert(id, format!(
          "{{ shareScope: {}, shareKey: {}, import: {}, requiredVersion: {}, strictVersion: {}, singleton: {}, eager: {}, fallback: {} }}",
          json_stringify(&data.share_scope),
          json_stringify(&data.share_key),
          json_stringify(&data.import),
          json_stringify(&data.required_version.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "*".to_string())),
          json_stringify(&data.strict_version),
          json_stringify(&data.singleton),
          json_stringify(&data.eager),
          data.fallback.as_deref().unwrap_or("undefined"),
        ));
      }
    };
    for chunk in chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey) {
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(
          &chunk,
          SourceType::ConsumeShared,
          &module_graph,
        );
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      let mut ids = vec![];
      for module in modules {
        add_module(module.identifier(), chunk, &mut ids);
      }
      if ids.is_empty() {
        continue;
      }
      chunk_to_module_mapping.insert(
        chunk
          .id
          .clone()
          .expect("should have chunkId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate"),
        ids,
      );
    }
    for chunk in chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey) {
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(
          &chunk,
          SourceType::ConsumeShared,
          &module_graph,
        );
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      for module in modules {
        add_module(module.identifier(), chunk, &mut initial_consumes);
      }
    }
    if module_id_to_consume_data_mapping.is_empty() {
      return Ok(RawSource::from("").boxed());
    }
    let module_id_to_consume_data_mapping = module_id_to_consume_data_mapping
      .into_iter()
      .map(|(k, v)| format!("{}: {}", json_stringify(&k), v))
      .collect::<Vec<_>>()
      .join(", ");
    let mut source = format!(
      r#"
__webpack_require__.consumesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToConsumeDataMapping: {{ {module_to_consume_data_mapping} }}, initialConsumes: {initial_consumes} }};
"#,
      chunk_mapping = json_stringify(&chunk_to_module_mapping),
      module_to_consume_data_mapping = module_id_to_consume_data_mapping,
      initial_consumes = json_stringify(&initial_consumes),
    );
    if self.enhanced {
      if compilation
        .chunk_graph
        .get_chunk_graph_chunk(&chunk_ukey)
        .runtime_requirements
        .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      {
        source += "__webpack_require__.f.consumes = function() { throw new Error(\"should have __webpack_require__.f.consumes\") }";
      }
      return Ok(RawSource::from(source).boxed());
    }
    source += include_str!("./consumesCommon.js");
    if !initial_consumes.is_empty() {
      source += include_str!("./consumesInitial.js");
    }
    if compilation
      .chunk_graph
      .get_chunk_graph_chunk(&chunk_ukey)
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      source += include_str!("./consumesLoading.js");
    }
    Ok(RawSource::from(source).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataConsumeShared {
  pub share_scope: String,
  pub share_key: String,
  pub import: Option<String>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
  pub fallback: Option<String>,
}

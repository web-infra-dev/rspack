use rspack_identifier::Identifier;
use rspack_sources::{BoxSource, RawSource, SourceExt};
use rustc_hash::FxHashMap;

use super::consume_shared_plugin::ConsumeVersion;
use crate::{
  impl_runtime_module, mf::utils::json_stringify, Chunk, ChunkUkey, Compilation, ModuleIdentifier,
  RuntimeGlobals, RuntimeModule, RuntimeModuleStage, SourceType,
};

#[derive(Debug, Eq)]
pub struct ConsumeSharedRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ConsumeSharedRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/consumes_loading"),
      chunk: None,
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

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
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
      if let Ok(code_gen) = compilation
        .code_generation_results
        .get(&module, Some(&chunk.runtime))
        && let Some(data) = code_gen.data.get::<CodeGenerationDataConsumeShared>()
      {
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
          &compilation.module_graph,
        );
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      let mut ids = vec![];
      for module in modules {
        add_module(module.identifier(), chunk, &mut ids);
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
          &compilation.module_graph,
        );
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      for module in modules {
        add_module(module.identifier(), chunk, &mut initial_consumes);
      }
    }
    if module_id_to_consume_data_mapping.is_empty() {
      return RawSource::from("").boxed();
    }
    let module_id_to_consume_data_mapping = module_id_to_consume_data_mapping
      .into_iter()
      .map(|(k, v)| {
        format!(
          "{}: {}",
          serde_json::to_string(&k)
            .expect("module_id_to_source_mapping key should able to json to_string"),
          v
        )
      })
      .collect::<Vec<_>>()
      .join(", ");
    let mut source = format!(
      r#"
__webpack_require__.MF.consumesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToConsumeDataMapping: {{ {module_to_consume_data_mapping} }}, initialConsumeModuleIds: {initial_consumes} }};
"#,
      chunk_mapping = serde_json::to_string(&chunk_to_module_mapping)
        .expect("chunk_to_module_mapping should able to json to_string"),
      module_to_consume_data_mapping = module_id_to_consume_data_mapping,
      initial_consumes = serde_json::to_string(&initial_consumes)
        .expect("initial_consumes should able to json to_string"),
    );
    if compilation
      .chunk_graph
      .get_chunk_graph_chunk(&chunk_ukey)
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      source += &format!("{ensure_chunk_handlers}.consumes = function(chunkId, promises) {{ return {consumes_loading_fn}(chunkId, promises); }};",
        ensure_chunk_handlers = RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
        consumes_loading_fn = "__webpack_require__.MF.consumesLoading",
      );
    }
    RawSource::from(source).boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(ConsumeSharedRuntimeModule);

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

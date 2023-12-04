use rspack_identifier::Identifier;
use rspack_sources::{BoxSource, RawSource, SourceExt};
use rustc_hash::FxHashMap;

use crate::{
  impl_runtime_module, Chunk, ChunkUkey, Compilation, ModuleIdentifier, RuntimeGlobals,
  RuntimeModule, RuntimeModuleStage, SourceType,
};

#[derive(Debug, Eq)]
pub struct ConsumeSharedRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ConsumeSharedRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/consumes"),
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
    let mut module_id_to_source_mapping = FxHashMap::default();
    let mut initial_consumes = Vec::new();
    let mut add_module = |module: ModuleIdentifier, chunk: &Chunk, ids: &mut Vec<String>| {
      let id = compilation
        .chunk_graph
        .get_module_id(module)
        .clone()
        .expect("should have moduleId at <ConsumeSharedRuntimeModule as RuntimeModule>::generate");
      ids.push(id.clone());
      if let Some(source) = compilation
        .code_generation_results
        .get(&module, Some(&chunk.runtime))
        .get(&SourceType::ConsumeShared)
      {
        module_id_to_source_mapping.insert(id, source.clone());
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
    if module_id_to_source_mapping.is_empty() {
      return RawSource::from("").boxed();
    }
    let module_to_handler_mapping = module_id_to_source_mapping
      .into_iter()
      .map(|(k, v)| {
        format!(
          "{}: {}",
          serde_json::to_string(&k)
            .expect("module_id_to_source_mapping key should able to json to_string"),
          v.source()
        )
      })
      .collect::<Vec<_>>()
      .join(", ");
    let mut source = format!(
      r#"
var chunkMapping = {chunk_mapping};
var moduleToHandlerMapping = {{ {module_to_handler_mapping} }};
var initialConsumes = {initial_consumes};
__webpack_require__.MF.initialConsumesData = {{ initialConsumes: initialConsumes, moduleToHandlerMapping: moduleToHandlerMapping }};
"#,
      chunk_mapping = serde_json::to_string(&chunk_to_module_mapping)
        .expect("chunk_to_module_mapping should able to json to_string"),
      module_to_handler_mapping = module_to_handler_mapping,
      initial_consumes = serde_json::to_string(&initial_consumes)
        .expect("initial_consumes should able to json to_string"),
    );
    if compilation
      .chunk_graph
      .get_chunk_graph_chunk(&chunk_ukey)
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      source += &format!("{ensure_chunk_handlers}.consumes = function(chunkId, promises) {{ return {consumes_loading_fn}({{ chunkId: chunkId, promises: promises, chunkMapping: chunkMapping, moduleToHandlerMapping: moduleToHandlerMapping }}); }};",
        ensure_chunk_handlers = RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
        consumes_loading_fn = "__webpack_require__.MF.consumes",
      );
    }
    RawSource::from(source).boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(ConsumeSharedRuntimeModule);

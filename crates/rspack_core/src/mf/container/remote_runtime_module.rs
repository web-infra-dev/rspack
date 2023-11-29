use rspack_identifier::{Identifiable, Identifier};
use rspack_sources::{BoxSource, RawSource, SourceExt};
use rustc_hash::FxHashMap;

use super::remote_module::RemoteModule;
use crate::{
  impl_runtime_module, ChunkUkey, Compilation, DependenciesBlock, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType,
};

#[derive(Debug, Eq)]
pub struct RemoteRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for RemoteRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/remotes_loading"),
      chunk: None,
    }
  }
}

impl RuntimeModule for RemoteRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <RemoteRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let mut chunk_to_remotes_mapping = FxHashMap::default();
    let mut id_to_external_and_name_mapping = FxHashMap::default();
    for chunk in chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey) {
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(
          &chunk,
          SourceType::Remote,
          &compilation.module_graph,
        );
      let mut remotes = Vec::new();
      for m in modules {
        let Some(m) = m.downcast_ref::<RemoteModule>() else {
          continue;
        };
        let name = m.internal_request.as_str();
        let id = compilation
          .chunk_graph
          .get_module_id(m.identifier())
          .as_deref()
          .expect("should have module_id at <RemoteRuntimeModule as RuntimeModule>::generate");
        let share_scope = m.share_scope.as_str();
        let dep = m.get_dependencies()[0];
        let external_module = compilation
          .module_graph
          .get_module(&dep)
          .expect("should have module");
        let external_module_id = compilation
          .chunk_graph
          .get_module_id(external_module.identifier())
          .as_deref()
          .expect("should have module_id at <RemoteRuntimeModule as RuntimeModule>::generate");
        remotes.push(id.to_string());
        id_to_external_and_name_mapping.insert(id, vec![share_scope, name, external_module_id]);
      }
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
      chunk_to_remotes_mapping.insert(
        chunk
          .id
          .as_ref()
          .expect("should have chunkId at <RemoteRuntimeModule as RuntimeModule>::generate"),
        remotes,
      );
    }
    RawSource::from(format!(
      r#"
var chunkMapping = {chunk_mapping};
__webpack_require__.MF.idToExternalAndNameMapping = {id_to_external_and_name_mapping};
{ensure_chunk_handlers}.remotes = function(chunkId, promises) {{ return {remotes_fn}({{ chunkId: chunkId, promises: promises, chunkMapping: chunkMapping, idToExternalAndNameMapping: __webpack_require__.MF.idToExternalAndNameMapping }}); }};
"#,
      chunk_mapping = serde_json::to_string(&chunk_to_remotes_mapping)
        .expect("chunk_to_remotes_mapping should able to json to_string"),
      id_to_external_and_name_mapping = serde_json::to_string(&id_to_external_and_name_mapping)
        .expect("id_to_external_and_name_mapping should able to json to_string"),
      ensure_chunk_handlers = RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
      remotes_fn = "__webpack_require__.MF.remotes",
    ))
    .boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(RemoteRuntimeModule);

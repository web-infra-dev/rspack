use std::sync::LazyLock;

use rspack_collections::Identifiable;
use rspack_core::{
  ChunkGraph, Compilation, DependenciesBlock, ModuleId, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, RuntimeTemplate, SourceType, impl_runtime_module,
};
use rspack_plugin_runtime::extract_runtime_globals_from_ejs;
use rustc_hash::FxHashMap;
use serde::Serialize;

use super::remote_module::RemoteModule;
use crate::utils::json_stringify;

static REMOTES_LOADING_TEMPLATE: &str = include_str!("./remotesLoading.ejs");
static REMOTES_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(REMOTES_LOADING_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct RemoteRuntimeModule {
  enhanced: bool,
}

impl RemoteRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, enhanced: bool) -> Self {
    Self::with_name(runtime_template, "remotes_loading", enhanced)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RemoteRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id.to_string(), REMOTES_LOADING_TEMPLATE.to_string())]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <RemoteRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);
    let mut chunk_to_remotes_mapping = FxHashMap::default();
    let mut id_to_remote_data_mapping = FxHashMap::default();
    let module_graph = compilation.get_module_graph();
    // Match enhanced/webpack behavior: include all referenced chunks so async ones are mapped too
    for chunk in
      chunk.get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    {
      let modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_by_source_type(&chunk, SourceType::Remote, module_graph);
      let mut remotes = Vec::new();
      for m in modules {
        let Some(m) = m.downcast_ref::<RemoteModule>() else {
          continue;
        };
        let name = m.internal_request.as_str();
        let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, m.identifier())
          .expect("should have module_id at <RemoteRuntimeModule as RuntimeModule>::generate");
        let share_scope = if self.enhanced {
          ShareScopeData::Multiple(m.share_scope.as_slice())
        } else {
          ShareScopeData::Single(m.share_scope.first().map_or("default", String::as_str))
        };
        let dep = m.get_dependencies()[0];
        let external_module = module_graph
          .get_module_by_dependency_id(&dep)
          .expect("should have module");
        let external_module_id = ChunkGraph::get_module_id(
          &compilation.module_ids_artifact,
          external_module.identifier(),
        )
        .expect("should have module_id at <RemoteRuntimeModule as RuntimeModule>::generate");
        remotes.push(id.to_string());
        id_to_remote_data_mapping.insert(
          id,
          RemoteData {
            share_scope,
            name,
            external_module_id,
            remote_name: &m.remote_key,
          },
        );
      }
      if remotes.is_empty() {
        continue;
      }
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk);
      chunk_to_remotes_mapping.insert(
        chunk
          .id()
          .expect("should have chunkId at <RemoteRuntimeModule as RuntimeModule>::generate"),
        remotes,
      );
    }

    let remotes_loading_impl = if self.enhanced {
      format!(
        "{ensure_chunk_handlers}.remotes = {ensure_chunk_handlers}.remotes || function() {{ throw new Error(\"should have {ensure_chunk_handlers}.remotes\"); }}",
        ensure_chunk_handlers = compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS),
      )
    } else {
      compilation
        .runtime_template
        .render(self.id.as_str(), None)?
    };
    Ok(format!(
      r#"
{require_name}.remotesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToRemoteDataMapping: {id_to_remote_data_mapping} }};
{remotes_loading_impl}
"#,
      require_name = compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      chunk_mapping = json_stringify(&chunk_to_remotes_mapping),
      id_to_remote_data_mapping = json_stringify(&id_to_remote_data_mapping),
      remotes_loading_impl = remotes_loading_impl,
    ))
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *REMOTES_LOADING_RUNTIME_REQUIREMENTS
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoteData<'a> {
  share_scope: ShareScopeData<'a>,
  name: &'a str,
  external_module_id: &'a ModuleId,
  remote_name: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ShareScopeData<'a> {
  Single(&'a str),
  Multiple(&'a [String]),
}

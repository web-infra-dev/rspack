use std::sync::LazyLock;

use rspack_collections::{Identifiable, IdentifierSet};
use rspack_core::{
  ChunkGraph, Compilation, DependenciesBlock, ModuleGraph, ModuleId, ModuleIdentifier,
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleRuntimeRequirements,
  RuntimeModuleStage, RuntimeTemplate, SourceType, impl_runtime_module,
};
use rspack_plugin_runtime::extract_runtime_globals_from_ejs;
use rustc_hash::FxHashMap;
use serde::Serialize;

use super::remote_module::RemoteModule;
use crate::{
  ShareScope,
  utils::{json_stringify, runtime_require_scope_name, runtime_require_scope_requirement},
};

static REMOTES_LOADING_TEMPLATE: &str = include_str!("./remotesLoading.ejs");
static REMOTES_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| RuntimeModuleRuntimeRequirements {
    force_context: RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE,
    ..extract_runtime_globals_from_ejs(REMOTES_LOADING_TEMPLATE)
  });

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
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let dependencies = REMOTES_LOADING_RUNTIME_REQUIREMENTS.dependencies
      | runtime_require_scope_requirement(compilation);
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      force_context: RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE,
      ..Default::default()
    }
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), REMOTES_LOADING_TEMPLATE.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunk_ukey = self
      .chunk()
      .expect("should have chunk in <RemoteRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&chunk_ukey);
    let mut chunk_to_remotes_mapping = FxHashMap::default();
    let mut id_to_remote_data_mapping = FxHashMap::default();
    let mut remote_key_to_remote_module_ids = FxHashMap::default();
    let mut remote_key_to_external_module_ids = FxHashMap::default();
    let mut remote_module_id_to_consumer_module_ids = FxHashMap::default();
    let mut consumer_module_id_to_parent_module_ids = FxHashMap::default();
    let mut remote_key_to_chunk_ids = FxHashMap::default();
    let module_graph = compilation.get_module_graph();
    let consumers = collect_consumers(compilation, module_graph);
    // Match enhanced/webpack behavior: include all referenced chunks so async ones are mapped too
    for chunk in
      chunk.get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    {
      let modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_by_source_type(&chunk, SourceType::Remote, module_graph);
      let mut remotes = Vec::new();
      let mut remote_keys = Vec::new();
      for m in modules {
        let Some(m) = m.downcast_ref::<RemoteModule>() else {
          continue;
        };
        let name = m.internal_request.as_str();
        let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, m.identifier())
          .expect("should have module_id at <RemoteRuntimeModule as RuntimeModule>::generate");
        let share_scope = match &m.share_scope {
          ShareScope::Single(s) => ShareScopeField::Single(s.as_str()),
          ShareScope::Multiple(v) => ShareScopeField::Multiple(v.as_slice()),
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
        let remote_key = m.remote_key.clone();
        let consumer_modules = consumers.get(&m.identifier()).cloned().unwrap_or_default();
        let consumer_module_ids = consumer_modules
          .iter()
          .map(|(_, module_id)| module_id.clone())
          .collect::<Vec<_>>();
        let mut pending_consumers = consumer_modules.clone();
        let mut visited_consumers = IdentifierSet::default();
        while let Some((consumer_identifier, consumer_id)) = pending_consumers.pop() {
          if !visited_consumers.insert(consumer_identifier) {
            continue;
          }
          let parents = consumers
            .get(&consumer_identifier)
            .cloned()
            .unwrap_or_default();
          add_to_mapping(
            &mut consumer_module_id_to_parent_module_ids,
            consumer_id,
            parents.iter().map(|(_, id)| id.clone()).collect(),
          );
          pending_consumers.extend(parents);
        }
        add_to_mapping(
          &mut remote_key_to_remote_module_ids,
          remote_key.clone(),
          vec![id.clone()],
        );
        add_to_mapping(
          &mut remote_key_to_external_module_ids,
          remote_key.clone(),
          vec![external_module_id.clone()],
        );
        add_to_mapping(
          &mut remote_module_id_to_consumer_module_ids,
          id.clone(),
          consumer_module_ids,
        );
        remote_keys.push(remote_key);
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
      let chunk_id = chunk
        .id()
        .expect("should have chunkId at <RemoteRuntimeModule as RuntimeModule>::generate");
      for remote_key in remote_keys {
        add_to_mapping(
          &mut remote_key_to_chunk_ids,
          remote_key,
          vec![chunk_id.clone()],
        );
      }
      chunk_to_remotes_mapping.insert(chunk_id, remotes);
    }
    sort_mapping_values(&mut remote_key_to_remote_module_ids);
    sort_mapping_values(&mut remote_key_to_external_module_ids);
    sort_mapping_values(&mut remote_module_id_to_consumer_module_ids);
    sort_mapping_values(&mut consumer_module_id_to_parent_module_ids);
    sort_mapping_values(&mut remote_key_to_chunk_ids);

    let remotes_loading_impl = if self.enhanced {
      format!(
        "{ensure_chunk_handlers}.remotes = {ensure_chunk_handlers}.remotes || function() {{ throw new Error(\"should have {ensure_chunk_handlers}.remotes\"); }}",
        ensure_chunk_handlers =
          runtime_template.render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS),
      )
    } else {
      runtime_template.render(self.id().as_str(), None)?
    };
    Ok(format!(
      r#"
{require_name}.remotesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToRemoteDataMapping: {id_to_remote_data_mapping}, remoteKeyToRemoteModuleIds: {remote_key_to_remote_module_ids}, remoteKeyToExternalModuleIds: {remote_key_to_external_module_ids}, remoteModuleIdToConsumerModuleIds: {remote_module_id_to_consumer_module_ids}, consumerModuleIdToParentModuleIds: {consumer_module_id_to_parent_module_ids}, remoteKeyToChunkIds: {remote_key_to_chunk_ids} }};
{remotes_loading_impl}
"#,
      require_name = runtime_require_scope_name(runtime_template),
      chunk_mapping = json_stringify(&chunk_to_remotes_mapping),
      id_to_remote_data_mapping = json_stringify(&id_to_remote_data_mapping),
      remote_key_to_remote_module_ids = json_stringify(&remote_key_to_remote_module_ids),
      remote_key_to_external_module_ids = json_stringify(&remote_key_to_external_module_ids),
      remote_module_id_to_consumer_module_ids =
        json_stringify(&remote_module_id_to_consumer_module_ids),
      consumer_module_id_to_parent_module_ids =
        json_stringify(&consumer_module_id_to_parent_module_ids),
      remote_key_to_chunk_ids = json_stringify(&remote_key_to_chunk_ids),
      remotes_loading_impl = remotes_loading_impl,
    ))
  }
}

fn add_to_mapping<K, V>(mapping: &mut FxHashMap<K, Vec<V>>, key: K, values: Vec<V>)
where
  K: std::hash::Hash + Eq,
{
  mapping.entry(key).or_default().extend(values);
}

fn sort_mapping_values<K, V>(mapping: &mut FxHashMap<K, Vec<V>>)
where
  K: std::hash::Hash + Eq,
  V: Ord,
{
  for values in mapping.values_mut() {
    values.sort_unstable();
    values.dedup();
  }
}

fn get_module_id(
  compilation: &Compilation,
  module_identifier: &ModuleIdentifier,
) -> Option<ModuleId> {
  ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier).cloned()
}

// A source module may be concatenated into several emitted modules. Its dependency
// IDs are shared, so incoming connections can retain only one rewritten origin.
// Reverse each emitted module's outgoing edges instead: every concatenation owner
// must be invalidated, even when the dependency's recorded origin is another owner.
fn collect_consumers(
  compilation: &Compilation,
  module_graph: &ModuleGraph,
) -> FxHashMap<ModuleIdentifier, Vec<(ModuleIdentifier, ModuleId)>> {
  let mut consumers = FxHashMap::<_, Vec<_>>::default();
  for (identifier, _) in module_graph.modules() {
    let Some(id) = get_module_id(compilation, identifier) else {
      continue;
    };
    for connection in module_graph.get_outgoing_connections(identifier) {
      consumers
        .entry(*connection.module_identifier())
        .or_default()
        .push((*identifier, id.clone()));
    }
  }
  for entries in consumers.values_mut() {
    entries.sort_unstable();
    entries.dedup();
  }
  consumers
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoteData<'a> {
  share_scope: ShareScopeField<'a>,
  name: &'a str,
  external_module_id: &'a ModuleId,
  remote_name: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ShareScopeField<'a> {
  Single(&'a str),
  Multiple(&'a [String]),
}

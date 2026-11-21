use std::sync::LazyLock;

use rspack_collections::{Identifiable, IdentifierMap, IdentifierSet};
use rspack_core::{
  ChunkGraph, ChunkUkey, Compilation, ConcatenatedModule, DependenciesBlock, ModuleGraph, ModuleId,
  ModuleIdentifier, NormalModule, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeModuleStage, RuntimeSpec, RuntimeTemplate, SourceType,
  impl_runtime_module,
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
  with_cache_data: bool,
}

impl RemoteRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, enhanced: bool, with_cache_data: bool) -> Self {
    Self::with_name(
      runtime_template,
      "remotes_loading",
      enhanced,
      with_cache_data,
    )
  }
}

pub fn should_emit_remote_cache_data(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  enhanced: bool,
) -> bool {
  enhanced
    && compilation.platform.is_node()
    && chunk_has_remotes(chunk_ukey, compilation)
    && has_node_cache_runtime(compilation.get_module_graph())
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
    let mut dependencies = REMOTES_LOADING_RUNTIME_REQUIREMENTS.dependencies
      | runtime_require_scope_requirement(compilation);
    if self.with_cache_data {
      dependencies |= RuntimeGlobals::MODULE_CACHE | RuntimeGlobals::MODULE_FACTORIES;
    }
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
    let mut module_id_to_parent_module_ids = FxHashMap::default();
    let module_graph = compilation.get_module_graph();
    let with_cache_data = self.with_cache_data;
    let consumers = if with_cache_data {
      collect_consumers(compilation, module_graph, chunk.runtime())
    } else {
      IdentifierMap::default()
    };
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
        let share_scope = match &m.share_scope {
          ShareScope::Single(s) => ShareScopeField::Single(s.as_str()),
          ShareScope::Multiple(v) => ShareScopeField::Multiple(v.as_slice()),
        };
        let dep = m
          .get_dependency_ids()
          .next()
          .expect("should have external dependency");
        let external_module = module_graph
          .get_module_by_dependency_id(dep)
          .expect("should have module");
        let external_module_id = ChunkGraph::get_module_id(
          &compilation.module_ids_artifact,
          external_module.identifier(),
        )
        .expect("should have module_id at <RemoteRuntimeModule as RuntimeModule>::generate");
        if with_cache_data {
          let mut pending = vec![(m.identifier(), id.clone())];
          let mut visited = IdentifierSet::default();
          while let Some((current_identifier, current_id)) = pending.pop() {
            if !visited.insert(current_identifier) {
              continue;
            }
            let parents = consumers
              .get(&current_identifier)
              .cloned()
              .unwrap_or_default();
            if !parents.is_empty() {
              add_to_mapping(
                &mut module_id_to_parent_module_ids,
                current_id,
                parents
                  .iter()
                  .map(|(_, parent_id)| parent_id.clone())
                  .collect(),
              );
              pending.extend(parents);
            }
          }
        }
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
      chunk_to_remotes_mapping.insert(chunk_id, remotes);
    }
    sort_mapping_values(&mut module_id_to_parent_module_ids);

    let remotes_loading_impl = if self.enhanced {
      format!(
        "{ensure_chunk_handlers}.remotes = {ensure_chunk_handlers}.remotes || function() {{ throw new Error(\"should have {ensure_chunk_handlers}.remotes\"); }}",
        ensure_chunk_handlers =
          runtime_template.render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS),
      )
    } else {
      runtime_template.render(self.id().as_str(), None)?
    };
    let require_name = runtime_require_scope_name(runtime_template);
    let cache_field = if with_cache_data {
      let module_cache = runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_CACHE);
      let module_factories =
        runtime_template.render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES);
      format!(
        ", cache: {{ moduleIdToParentModuleIds: {}, modules: {module_cache}, factories: {module_factories} }}",
        json_stringify(&module_id_to_parent_module_ids),
      )
    } else {
      String::new()
    };
    Ok(format!(
      r#"
{require_name}.remotesLoadingData = {{ chunkMapping: {chunk_mapping}, moduleIdToRemoteDataMapping: {id_to_remote_data_mapping}{cache_field} }};
{remotes_loading_impl}
"#,
      chunk_mapping = json_stringify(&chunk_to_remotes_mapping),
      id_to_remote_data_mapping = json_stringify(&id_to_remote_data_mapping),
      remotes_loading_impl = remotes_loading_impl,
    ))
  }
}

fn chunk_has_remotes(chunk_ukey: &ChunkUkey, compilation: &Compilation) -> bool {
  let Some(chunk) = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_ukey)
  else {
    return false;
  };
  let module_graph = compilation.get_module_graph();
  chunk
    .get_all_referenced_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    .into_iter()
    .any(|referenced| {
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .has_chunk_module_by_source_type(&referenced, SourceType::Remote, module_graph)
    })
}

fn is_node_cache_specifier(specifier: &str) -> bool {
  specifier.contains("@module-federation/node")
    || specifier.contains("webpack-bundler-runtime/clear-cache")
    || specifier.contains("webpack-bundler-runtime/dist/clear-cache")
    || specifier.contains("webpack-bundler-runtime\\dist\\clear-cache")
}

fn has_node_cache_runtime(module_graph: &ModuleGraph) -> bool {
  module_graph.modules().any(|(identifier, module)| {
    if is_node_cache_specifier(identifier.as_str()) {
      return true;
    }
    if let Some(concatenated) = module
      .as_ref()
      .as_any()
      .downcast_ref::<ConcatenatedModule>()
    {
      return concatenated
        .get_modules()
        .iter()
        .any(|inner| is_node_cache_specifier(inner.id.as_str()));
    }
    let Some(normal_module) = module.as_ref().as_any().downcast_ref::<NormalModule>() else {
      return false;
    };
    is_node_cache_specifier(normal_module.raw_request())
      || is_node_cache_specifier(normal_module.user_request())
      || normal_module
        .resource_resolved_data()
        .description()
        .and_then(|data| data.json().get("name"))
        .and_then(|name| name.as_str())
        == Some("@module-federation/node")
  })
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
// Reverse each emitted module's active outgoing edges instead: every concatenation
// owner must be invalidated, even when the dependency's recorded origin is another owner.
fn collect_consumers(
  compilation: &Compilation,
  module_graph: &ModuleGraph,
  runtime: &RuntimeSpec,
) -> IdentifierMap<Vec<(ModuleIdentifier, ModuleId)>> {
  let mut consumers = IdentifierMap::<Vec<_>>::default();
  for (identifier, _) in module_graph.modules() {
    let Some(id) = get_module_id(compilation, identifier) else {
      continue;
    };
    for connection in module_graph.get_outgoing_connections(identifier) {
      if !connection.is_target_active(
        module_graph,
        Some(runtime),
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &compilation.exports_info_artifact,
      ) {
        continue;
      }
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

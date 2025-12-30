use indoc::formatdoc;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkGraph, ChunkUkey, Compilation, Module, ModuleGraph, ModuleGraphRef, ModuleId,
  ModuleIdentifier, RuntimeModule, RuntimeModuleStage, impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::{
  constants::LAYERS_NAMES,
  loaders::action_entry_loader::{ACTION_ENTRY_LOADER_IDENTIFIER, parse_action_entries},
  plugin_state::PLUGIN_STATE_BY_COMPILER_ID,
  reference_manifest::{ManifestExport, ManifestNode, ModuleLoading, ServerReferenceManifest},
  utils::{ChunkModules, to_json_string_literal},
};

fn serialize_none_as_empty_object<S, T>(val: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  T: Serialize,
{
  match val {
    Some(v) => v.serialize(serializer),
    None => {
      let map = serializer.serialize_map(Some(0))?;
      map.end()
    }
  }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RscManifest<'a> {
  pub server_manifest: &'a FxHashMap<String, ManifestExport>,
  pub client_manifest: &'a FxHashMap<String, ManifestExport>,
  pub server_consumer_module_map: &'a FxHashMap<String, ManifestNode>,
  pub module_loading: &'a ModuleLoading,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_css_files: Option<&'a FxHashMap<String, FxIndexSet<String>>>,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_js_files: Option<&'a FxIndexSet<String>>,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct RscManifestRuntimeModule {
  id: Identifier,
  chunk_ukey: Option<ChunkUkey>,
}

impl RscManifestRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/rsc_manifest"), None)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RscManifestRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let server_compiler_id = compilation.compiler_id();

    let Some(entry_name) = self
      .chunk_ukey
      .as_ref()
      .and_then(|chunk_ukey| compilation.chunk_by_ukey.get(chunk_ukey))
      .and_then(|chunk| chunk.get_entry_options(&compilation.chunk_group_by_ukey))
      .and_then(|entry_options| entry_options.name.as_ref())
    else {
      return Ok(String::new());
    };

    let mut state_by_compiler_id = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
    let plugin_state = state_by_compiler_id
      .get_mut(&server_compiler_id)
      .ok_or_else(|| {
        rspack_error::error!(
          "Failed to find RSC plugin state for compiler (ID: {}).",
          server_compiler_id.as_u32()
        )
      })?;

    build_server_manifest(compilation, &mut plugin_state.server_actions)?;
    let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
      rspack_error::error!(
        "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
      )
    })?;
    let server_consumer_module_map =
      build_server_consumer_module_map(compilation, &plugin_state.client_modules);

    let rsc_manifest = RscManifest {
      server_manifest: &plugin_state.server_actions,
      client_manifest: &plugin_state.client_modules,
      server_consumer_module_map: &server_consumer_module_map,
      module_loading,
      entry_css_files: plugin_state.entry_css_files.get(entry_name),
      entry_js_files: plugin_state.entry_js_files.get(entry_name),
    };

    Ok(formatdoc! {
      r#"
        __webpack_require__.rscM = JSON.parse({});
      "#,
      to_json_string_literal(&rsc_manifest).to_rspack_result()?,
    })
  }

  fn attach(&mut self, chunk_ukey: ChunkUkey) {
    self.chunk_ukey = Some(chunk_ukey);
  }
}

fn build_server_manifest(
  compilation: &Compilation,
  server_actions: &mut ServerReferenceManifest,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  for module in module_graph.modules().values() {
    let module_id =
      match ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier()) {
        Some(id) => id,
        None => continue,
      };

    let Some(normal_module) = module.as_normal_module() else {
      continue;
    };

    let request = normal_module.request();
    if !request.starts_with(ACTION_ENTRY_LOADER_IDENTIFIER) {
      continue;
    }

    let loader_query = request
      .split_once('?')
      .map(|x| x.1)
      .unwrap_or_default()
      .rsplit_once('!')
      .map(|x| x.0)
      .unwrap_or_default();
    let loader_options = form_urlencoded::parse(loader_query.as_bytes());

    for (k, v) in loader_options {
      if k == "actions" {
        if let Some(actions) = parse_action_entries(v.into_owned())? {
          for action in actions {
            server_actions.insert(
              action.id.to_string(),
              ManifestExport {
                id: module_id.to_string(),
                name: action.id.to_string(),
                // Server Action modules serve as endpoints rather than code splitting points, so ensuring chunk loading at runtime is unnecessary.
                chunks: vec![],
                r#async: Some(ModuleGraph::is_async(
                  &compilation.async_modules_artifact.borrow(),
                  &module.identifier(),
                )),
              },
            );
          }
        }
        break;
      }
    }
  }

  Ok(())
}

fn record_module(
  compilation: &Compilation,
  client_modules: &FxHashMap<String, ManifestExport>,
  module_graph: &ModuleGraphRef<'_>,
  module_idenfitifier: ModuleIdentifier,
  module_id: ModuleId,
  server_consumer_module_map: &mut FxHashMap<String, ManifestNode>,
) {
  let Some(module) = module_graph.module_by_identifier(&module_idenfitifier) else {
    return;
  };
  let Some(normal_module) = module.as_normal_module() else {
    return;
  };

  if normal_module.build_info().rsc.as_ref().is_none()
    || normal_module
      .get_layer()
      .is_none_or(|layer| layer != LAYERS_NAMES.server_side_rendering)
  {
    return;
  }

  // Match Resource is undefined unless an import is using the inline match resource syntax
  // https://webpack.js.org/api/loaders/#inline-matchresource
  let mod_path = normal_module
    .match_resource()
    .map(|resource| resource.path())
    .unwrap_or(normal_module.resource_resolved_data().path());
  let mod_query = normal_module.resource_resolved_data().query().unwrap_or("");
  // query is already part of mod.resource
  // so it's only necessary to add it for matchResource or mod.resourceResolveData
  let resource = match mod_path {
    Some(mod_path) => format!("{}{}", mod_path.as_str(), mod_query),
    None => normal_module
      .resource_resolved_data()
      .resource()
      .to_string(),
  };

  if resource.is_empty() {
    return;
  }

  let manifest_export = ManifestExport {
    id: module_id.to_string(),
    name: "*".to_string(),
    chunks: vec![],
    r#async: Some(ModuleGraph::is_async(
      &compilation.async_modules_artifact.borrow(),
      &module_idenfitifier,
    )),
  };
  let mut node = FxHashMap::default();
  node.insert("*".to_string(), manifest_export);
  if let Some(export) = client_modules.get(&resource) {
    server_consumer_module_map.insert(export.id.clone(), node);
  }
}

fn build_server_consumer_module_map(
  compilation: &Compilation,
  client_modules: &FxHashMap<String, ManifestExport>,
) -> FxHashMap<String, ManifestNode> {
  let mut server_consumer_module_map: FxHashMap<String, ManifestNode> = Default::default();
  let module_graph = compilation.get_module_graph();
  let chunk_modules = ChunkModules::new(compilation, &module_graph);
  for (module_identifier, module_id) in chunk_modules {
    record_module(
      compilation,
      client_modules,
      &module_graph,
      module_identifier,
      module_id,
      &mut server_consumer_module_map,
    );
  }
  server_consumer_module_map
}

#![allow(clippy::ref_option_ref)]

use indoc::formatdoc;
use rspack_core::{
  ChunkGraph, Compilation, ModuleGraph, ModuleId, ModuleIdentifier, ModuleType, RuntimeGlobals,
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::{
  constants::LAYERS_NAMES,
  loaders::action_entry_loader::{ACTION_ENTRY_LOADER_IDENTIFIER, parse_action_entries},
  plugin_state::{PLUGIN_STATES, PluginState},
  reference_manifest::{
    ActionReferenceManifest, ActionReferenceManifestEntry, ClientReferenceManifest, ManifestExport,
    ManifestNode, ModuleLoading, ServerReferenceManifest,
  },
  utils::{get_canonical_module_resource, is_federation_virtual_module, to_json_string_literal},
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
  pub version: u8,
  pub server_manifest: &'a FxHashMap<String, ManifestExport>,
  pub client_manifest: &'a FxHashMap<String, ManifestExport>,
  #[serde(default, skip_serializing_if = "FxHashMap::is_empty")]
  pub client_references: &'a ClientReferenceManifest,
  #[serde(default, skip_serializing_if = "FxHashMap::is_empty")]
  pub actions: &'a ActionReferenceManifest,
  pub server_consumer_module_map: &'a FxHashMap<String, ManifestNode>,
  pub module_loading: &'a ModuleLoading,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_css_files: Option<&'a FxHashMap<String, FxIndexSet<String>>>,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_js_files: Option<&'a FxIndexSet<String>>,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct RscManifestRuntimeModule {}

impl RscManifestRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RscManifestRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let server_compiler_id = compilation.compiler_id();

    let Some(entry_name) = self
      .chunk
      .as_ref()
      .and_then(|chunk_ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(chunk_ukey)
      })
      .and_then(|chunk| {
        chunk.get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      })
      .and_then(|entry_options| entry_options.name.as_ref())
    else {
      return Ok(String::new());
    };

    let mut plugin_state = PLUGIN_STATES.get_mut(&server_compiler_id).ok_or_else(|| {
      rspack_error::error!(
        "Failed to find RSC plugin state for compiler (ID: {}).",
        server_compiler_id.as_u32()
      )
    })?;

    let PluginState {
      server_actions,
      action_references,
      ..
    } = &mut *plugin_state;
    build_server_manifest(compilation, server_actions, action_references)?;
    let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
      rspack_error::error!(
        "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
      )
    })?;
    let server_consumer_module_map =
      build_server_consumer_module_map(compilation, &plugin_state.client_modules);

    let rsc_manifest = RscManifest {
      version: 1,
      server_manifest: &plugin_state.server_actions,
      client_manifest: &plugin_state.client_modules,
      client_references: &plugin_state.client_references,
      actions: &plugin_state.action_references,
      server_consumer_module_map: &server_consumer_module_map,
      module_loading,
      entry_css_files: plugin_state.entry_css_files.get(entry_name),
      entry_js_files: plugin_state.entry_js_files.get(entry_name),
    };

    Ok(formatdoc! {
      r#"
        {require_name}.rscM = JSON.parse({rsc_manifest_json});
      "#,
      require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
      rsc_manifest_json = to_json_string_literal(&rsc_manifest).to_rspack_result()?,
    })
  }
}

fn build_server_manifest(
  compilation: &Compilation,
  server_actions: &mut ServerReferenceManifest,
  action_references: &mut ActionReferenceManifest,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let mut record_module = |module_identifier: &ModuleIdentifier,
                           module_id: &ModuleId|
   -> Result<()> {
    let Some(module) = module_graph.module_by_identifier(module_identifier) else {
      return Ok(());
    };
    let Some(normal_module) = module.as_normal_module() else {
      return Ok(());
    };

    let request = normal_module.request();
    if !request.starts_with(ACTION_ENTRY_LOADER_IDENTIFIER) {
      return Ok(());
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
      if k != "actions" {
        continue;
      }

      if let Some(actions) = parse_action_entries(v.into_owned())? {
        for action in actions {
          let manifest_export = ManifestExport {
            id: module_id.to_string(),
            name: action.id.clone(),
            // Server Action modules serve as endpoints rather than code splitting points,
            // so ensuring chunk loading at runtime is unnecessary.
            chunks: vec![],
            r#async: Some(ModuleGraph::is_async(
              &compilation.async_modules_artifact,
              &module.identifier(),
            )),
          };
          if let Some(existing) = server_actions.get(&action.id)
            && (existing.id != manifest_export.id
              || existing.name != manifest_export.name
              || existing.r#async != manifest_export.r#async)
          {
            return Err(rspack_error::error!(
              "Conflicting server action id \"{}\" resolved to multiple modules (\"{}\" and \"{}\").",
              action.id,
              existing.id,
              manifest_export.id
            ));
          }
          server_actions.insert(action.id.clone(), manifest_export);

          let action_reference = ActionReferenceManifestEntry {
            local_action_id: action.id.clone(),
            export_name: Some(action.exported_name.clone()),
            module_resource: Some(action.path.to_string()),
          };
          if let Some(existing) = action_references.get(&action.id)
            && existing != &action_reference
          {
            return Err(rspack_error::error!(
              "Conflicting RSC action metadata for action id \"{}\".",
              action.id
            ));
          }
          action_references.insert(action.id.clone(), action_reference);
        }
      }
      break;
    }

    Ok(())
  };

  for (_, module) in module_graph.modules() {
    let module_id =
      match ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier()) {
        Some(id) => id,
        None => continue,
      };

    if let Some(concatenated_module) = module.as_concatenated_module() {
      for inner_module in concatenated_module.get_modules() {
        let inner_module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, inner_module.id)
            .unwrap_or(module_id);
        record_module(&inner_module.id, inner_module_id)?;
      }
      continue;
    }

    record_module(&module.identifier(), module_id)?;
  }

  Ok(())
}

fn build_server_consumer_module_map(
  compilation: &Compilation,
  client_modules: &FxHashMap<String, ManifestExport>,
) -> FxHashMap<String, ManifestNode> {
  let mut server_consumer_module_map: FxHashMap<String, ManifestNode> = Default::default();
  let module_graph = compilation.get_module_graph();
  let mut client_exports_by_resource: FxHashMap<String, Vec<&ManifestExport>> = Default::default();
  let mut rsc_server_modules_by_resource: FxHashMap<String, (u8, ManifestExport)> =
    Default::default();
  let mut ssr_server_modules_by_resource: FxHashMap<String, (u8, ManifestExport)> =
    Default::default();

  for (resource_key, export) in client_modules {
    let normalized_resource = match resource_key.split_once('#') {
      Some((resource, _)) => resource.to_string(),
      None => resource_key.clone(),
    };
    client_exports_by_resource
      .entry(normalized_resource)
      .or_default()
      .push(export);
  }

  let mut record_module = |module_identifier: &ModuleIdentifier, module_id: &ModuleId| {
    let Some(module) = module_graph.module_by_identifier(module_identifier) else {
      return;
    };
    let is_consume_shared_module = matches!(module.module_type(), ModuleType::ConsumeShared);
    if is_federation_virtual_module(module.as_ref()) && !is_consume_shared_module {
      return;
    }
    enum LayerKind {
      Rsc,
      Ssr,
    }
    let layer_kind = if module
      .get_layer()
      .is_some_and(|layer| layer == LAYERS_NAMES.react_server_components)
    {
      LayerKind::Rsc
    } else if module
      .get_layer()
      .is_some_and(|layer| layer == LAYERS_NAMES.server_side_rendering)
    {
      LayerKind::Ssr
    } else {
      return;
    };
    let module_kind_priority = if is_consume_shared_module { 0u8 } else { 1u8 };

    let resource = get_canonical_module_resource(compilation, module.as_ref());
    if resource.is_empty() {
      return;
    }
    if !client_exports_by_resource.contains_key(&resource) {
      return;
    }

    let mut required_chunks = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(module.identifier())
      .iter()
      .filter_map(|chunk_ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(chunk_ukey)
          .and_then(|chunk| {
            let has_js_asset = chunk.files().iter().any(|file| {
              file.ends_with(".js") || file.ends_with(".mjs") || file.ends_with(".cjs")
            });
            if has_js_asset { Some(chunk) } else { None }
          })
          .and_then(|chunk| chunk.id().map(|id| id.to_string()))
      })
      .collect::<Vec<_>>();
    required_chunks.sort();
    required_chunks.dedup();

    let manifest_export = ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: required_chunks,
      r#async: Some(ModuleGraph::is_async(
        &compilation.async_modules_artifact,
        &module.identifier(),
      )),
    };

    match layer_kind {
      LayerKind::Rsc => match rsc_server_modules_by_resource.get(&resource) {
        Some((existing_kind_priority, _)) if *existing_kind_priority <= module_kind_priority => {}
        _ => {
          rsc_server_modules_by_resource.insert(resource, (module_kind_priority, manifest_export));
        }
      },
      LayerKind::Ssr => match ssr_server_modules_by_resource.get(&resource) {
        Some((existing_kind_priority, _)) if *existing_kind_priority <= module_kind_priority => {}
        _ => {
          ssr_server_modules_by_resource.insert(resource, (module_kind_priority, manifest_export));
        }
      },
    }
  };

  for (_, module) in module_graph.modules() {
    let Some(module_id) =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
    else {
      continue;
    };
    if let Some(concatenated_module) = module.as_concatenated_module() {
      for inner_module in concatenated_module.get_modules() {
        let inner_module_id =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, inner_module.id)
            .unwrap_or(module_id);
        record_module(&inner_module.id, inner_module_id);
      }
    } else {
      record_module(&module.identifier(), module_id);
    }
  }

  for (resource, client_exports) in &client_exports_by_resource {
    let rsc_server_module_export = rsc_server_modules_by_resource
      .get(resource)
      .map(|(_, export)| export.clone());
    let ssr_server_module_export = ssr_server_modules_by_resource
      .get(resource)
      .map(|(_, export)| export.clone());
    let Some(default_server_module_export) = ssr_server_module_export
      .clone()
      .or(rsc_server_module_export.clone())
    else {
      continue;
    };
    let build_node = |server_module_export: &ManifestExport| {
      let mut node = FxHashMap::default();
      node.insert("*".to_string(), server_module_export.clone());
      for export in client_exports {
        if export.name == "*" || export.name == "__esModule" {
          continue;
        }
        node.insert(
          export.name.clone(),
          ManifestExport {
            id: server_module_export.id.clone(),
            name: export.name.clone(),
            chunks: server_module_export.chunks.clone(),
            r#async: server_module_export.r#async,
          },
        );
      }
      node
    };

    let default_node = build_node(&default_server_module_export);
    let rsc_node = rsc_server_module_export
      .as_ref()
      .map_or_else(|| default_node.clone(), build_node);
    let ssr_node = ssr_server_module_export
      .as_ref()
      .map_or_else(|| default_node.clone(), build_node);

    let client_module_ids = client_exports
      .iter()
      .map(|export| export.id.clone())
      .collect::<FxHashSet<_>>();
    for client_module_id in client_module_ids {
      server_consumer_module_map.insert(client_module_id.clone(), default_node.clone());

      let ssr_layer_prefix = format!("({})/", LAYERS_NAMES.server_side_rendering);
      if !client_module_id.starts_with(&ssr_layer_prefix) {
        server_consumer_module_map.insert(
          format!("{ssr_layer_prefix}{client_module_id}"),
          ssr_node.clone(),
        );
      }

      let rsc_layer_prefix = format!("({})/", LAYERS_NAMES.react_server_components);
      if !client_module_id.starts_with(&rsc_layer_prefix) {
        server_consumer_module_map.insert(
          format!("{rsc_layer_prefix}{client_module_id}"),
          rsc_node.clone(),
        );
      }
    }
  }
  server_consumer_module_map
}

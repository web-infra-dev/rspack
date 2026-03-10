use std::sync::Arc;

use rspack_core::{ChunkGraph, Compilation, Module, ModuleGraph, ModuleId, ModuleIdentifier};
use rspack_error::Result;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
  constants::LAYERS_NAMES,
  loaders::action_entry_loader::{ACTION_ENTRY_LOADER_IDENTIFIER, parse_action_entries},
  utils::{ChunkModules, get_module_resource},
};

#[derive(Debug, Clone, Serialize)]
pub struct ManifestExport {
  /// Rspack module id
  pub id: String,
  /// Export name
  pub name: String,
  /// Chunks for the module. JS and CSS.
  pub chunks: Vec<String>,
  /// If chunk contains async module
  #[serde(skip_serializing_if = "Option::is_none")]
  pub r#async: Option<bool>,
}

pub type ManifestNode = FxHashMap<String, ManifestExport>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrossOriginMode {
  #[serde(rename = "use-credentials")]
  UseCredentials,
  #[serde(rename = "")]
  Anonymous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLoading {
  pub prefix: String,
  #[serde(rename = "crossOrigin")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cross_origin: Option<CrossOriginMode>,
}

pub type ServerReferenceManifest = FxHashMap<String, ManifestExport>;

/// Fills each entry's `server_actions` in `entries` by resolving action-loader modules
/// to their entries via the chunk graph.
pub fn build_server_manifest(
  compilation: &Compilation,
  entries: &mut FxHashMap<Arc<str>, crate::plugin_state::EntryState>,
) -> Result<()> {
  let mut server_actions_per_entry: FxHashMap<Arc<str>, ServerReferenceManifest> =
    FxHashMap::default();
  let module_graph = compilation.get_module_graph();
  let artifact = &compilation.build_chunk_graph_artifact;
  let chunk_graph = &artifact.chunk_graph;

  // Build module_identifier -> list of entry names (by walking entrypoints -> chunks -> modules).
  let mut module_to_entries: FxHashMap<ModuleIdentifier, Vec<Arc<str>>> = FxHashMap::default();
  for (entry_name, group_ukey) in &artifact.entrypoints {
    let entry_name: Arc<str> = Arc::from(entry_name.to_string());

    let Some(group) = artifact.chunk_group_by_ukey.get(group_ukey) else {
      continue;
    };
    for chunk_ukey in &group.chunks {
      for module_identifier in chunk_graph.get_chunk_modules_identifier(chunk_ukey).iter() {
        module_to_entries
          .entry(*module_identifier)
          .or_default()
          .push(entry_name.clone());
      }
    }
  }
  for entries in module_to_entries.values_mut() {
    entries.sort();
    entries.dedup();
  }

  let mut record_module =
    |module_identifier: &ModuleIdentifier, module_id: &ModuleId| -> Result<()> {
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
          let entry_names = module_to_entries
            .get(module_identifier)
            .cloned()
            .unwrap_or_default();
          let is_async =
            ModuleGraph::is_async(&compilation.async_modules_artifact, &module.identifier());
          for action in actions {
            let export = ManifestExport {
              id: module_id.to_string(),
              name: action.id.clone(),
              chunks: vec![],
              r#async: Some(is_async),
            };
            for entry_name in &entry_names {
              server_actions_per_entry
                .entry(entry_name.clone())
                .or_default()
                .insert(action.id.clone(), export.clone());
            }
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
        record_module(&inner_module.id, module_id)?;
      }
      continue;
    }

    record_module(&module.identifier(), module_id)?;
  }

  for (entry_name, manifest) in server_actions_per_entry {
    entries.entry(entry_name).or_default().server_actions = manifest;
  }
  Ok(())
}

pub fn build_server_consumer_module_map(
  compilation: &Compilation,
  client_modules: &FxHashMap<String, ManifestExport>,
) -> FxHashMap<String, ManifestNode> {
  let mut server_consumer_module_map: FxHashMap<String, ManifestNode> = Default::default();
  let module_graph = compilation.get_module_graph();
  let chunk_modules = ChunkModules::new(compilation, module_graph);

  let mut record_module = |module_identifier: &ModuleIdentifier, module_id: &ModuleId| {
    let Some(module) = module_graph.module_by_identifier(module_identifier) else {
      return;
    };
    let Some(normal_module) = module.as_normal_module() else {
      return;
    };

    if normal_module
      .get_layer()
      .is_none_or(|layer| layer != LAYERS_NAMES.server_side_rendering)
    {
      return;
    }

    let resource = get_module_resource(module.as_ref());
    if resource.is_empty() {
      return;
    }

    let manifest_export = ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: vec![],
      r#async: Some(ModuleGraph::is_async(
        &compilation.async_modules_artifact,
        &module.identifier(),
      )),
    };

    let mut node = FxHashMap::default();
    node.insert("*".to_string(), manifest_export);

    if let Some(export) = client_modules.get(resource.as_ref()) {
      server_consumer_module_map.insert(export.id.clone(), node);
    }
  };

  for (module_identifier, module_id) in chunk_modules {
    let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
      continue;
    };

    if let Some(concatenated_module) = module.as_concatenated_module() {
      for inner_module in concatenated_module.get_modules() {
        record_module(&inner_module.id, &module_id);
      }
    } else {
      record_module(&module_identifier, &module_id);
    }
  }
  server_consumer_module_map
}

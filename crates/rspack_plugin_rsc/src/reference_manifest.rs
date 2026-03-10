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

pub fn build_server_manifest(
  compilation: &Compilation,
  server_actions: &mut ServerReferenceManifest,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

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
          for action in actions {
            server_actions.insert(
              action.id.clone(),
              ManifestExport {
                id: module_id.to_string(),
                name: action.id.clone(),
                // Server Action modules serve as endpoints rather than code splitting points,
                // so ensuring chunk loading at runtime is unnecessary.
                chunks: vec![],
                r#async: Some(ModuleGraph::is_async(
                  &compilation.async_modules_artifact,
                  &module.identifier(),
                )),
              },
            );
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

use std::sync::{Arc, LazyLock};

use regex::Regex;
use rspack_collections::{Identifiable, IdentifierSet};
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkUkey,
  ClientEntryType, Compilation, CompilationProcessAssets, CompilerFinishMake, Dependency,
  DependencyId, EntryDependency, EntryOptions, ExportsInfoGetter, GroupOptions, Logger, Module,
  ModuleId, ModuleIdentifier, ModuleType, NormalModule, Plugin, PrefetchExportsInfoMode, RSCMeta,
  RSCModuleType, RuntimeSpec,
  build_module_graph::{UpdateParam, update_module_graph},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use swc_core::atoms::Wtf8Atom;

use crate::{
  client_reference_dependency::ClientReferenceDependency,
  constants::WEBPACK_LAYERS,
  utils::{EntryModules, ServerEntries},
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

type ManifestNode = FxHashMap<String, ManifestExport>;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "inlined")]
pub enum CssResource {
  #[serde(rename = "true")]
  Inlined { path: String, content: String },
  #[serde(rename = "false")]
  Uninlined { path: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientReferenceManifest {
  #[serde(rename = "clientModules")]
  pub client_modules: ManifestNode,
  #[serde(rename = "rscModuleMapping")]
  pub rsc_module_mapping: FxHashMap<String, ManifestNode>,

  #[serde(rename = "moduleLoading")]
  pub module_loading: ModuleLoading,
  #[serde(rename = "ssrModuleMapping")]
  pub ssr_module_mapping: FxHashMap<String, ManifestNode>,
  #[serde(rename = "entryCSSFiles")]
  pub entry_css_files: FxHashMap<String, Vec<CssResource>>,
  #[serde(rename = "entryJSFiles")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entry_js_files: Option<FxHashMap<String, Vec<String>>>,
}

fn record_module(
  mod_id: ModuleId,
  module_identifier: &ModuleIdentifier,
  context: &str,
  manifest: &mut ClientReferenceManifest,
  // plugin_state: &PluginState,
  compilation: &Compilation,
  required_chunks: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
  let Some(module) = compilation.module_by_identifier(module_identifier) else {
    return Ok(());
  };
  let Some(normal_module) = module.as_normal_module() else {
    return Ok(());
  };

  let resource = if normal_module.module_type().as_str() == "css/mini-extract" {
    let identifier = normal_module.identifier();
    if let Some(pos) = identifier.rfind('!') {
      &identifier[pos + 1..]
    } else {
      identifier.as_str()
    }
  } else {
    normal_module.resource_resolved_data().resource()
  };
  if resource.is_empty() {
    return Ok(());
  }

  // Note that this isn't that reliable as webpack is still possible to assign
  // additional queries to make sure there's no conflict even using the `named`
  // module ID strategy.
  compilation.options.context.as_path().relative();
  let mut ssr_named_module_id = relative(
    context,
    module
      .resource_resolve_data
      .as_ref()
      .and_then(|d| d.path.as_ref())
      .unwrap_or(&resource),
  );

  let rsc_named_module_id = relative(
    context,
    module
      .resource_resolve_data
      .as_ref()
      .and_then(|d| d.path.as_ref())
      .unwrap_or(&resource),
  );

  if !ssr_named_module_id.starts_with('.') {
    ssr_named_module_id = format!("./{}", ssr_named_module_id.replace('\\', "/"));
  }

  // The client compiler will always use the CJS Next.js build, so here we
  // also add the mapping for the ESM build (Edge runtime) to consume.
  let next_dist_regex = Regex::new(r"[\\/]next[\\/]dist[\\/]")?;
  let esm_resource = if next_dist_regex.is_match(&resource) {
    let esm_replacement = format!("/next/dist/esm/{}", MAIN_SEPARATOR);
    Some(
      next_dist_regex
        .replace(&resource, esm_replacement.as_str())
        .to_string(),
    )
  } else {
    None
  };

  let (mut final_ssr_named_module_id, mut final_resource) = (ssr_named_module_id, resource);

  // An extra query param is added to the resource key when it's optimized
  // through the Barrel Loader. That's because the same file might be created
  // as multiple modules (depending on what you import from it).
  // See also: webpack/loaders/next-flight-loader/index.ts.
  if let Some(match_res) = &module.match_resource {
    if match_res.starts_with(BARREL_OPTIMIZATION_PREFIX) {
      final_ssr_named_module_id =
        format_barrel_optimized_resource(&final_ssr_named_module_id, match_res);
      final_resource = format_barrel_optimized_resource(&final_resource, match_res);
    }
  }

  // addClientReference
  {
    let is_async =
            // compilation.moduleGraph.isAsync(mod) || // 需要实现
            plugin_state.ssr_modules.get(&final_ssr_named_module_id)
                .map(|m| m.r#async)
                .unwrap_or(false) ||
            plugin_state.edge_ssr_modules.get(&final_ssr_named_module_id)
                .map(|m| m.r#async)
                .unwrap_or(false);

    let export_name = final_resource.clone();
    let manifest_entry = ManifestEntry {
      id: mod_id.clone(),
      name: "*".to_string(),
      chunks: required_chunks.to_vec(),
      r#async: is_async,
    };

    manifest
      .client_modules
      .insert(export_name.clone(), manifest_entry.clone());

    if let Some(esm_res) = &esm_resource {
      manifest
        .client_modules
        .insert(esm_res.clone(), manifest_entry);
    }
  }

  // addSSRIdMapping
  {
    let export_name = &final_resource;

    if let Some(module_info) = plugin_state.ssr_modules.get(&final_ssr_named_module_id) {
      let mod_id_str = mod_id.to_string();
      let mapping = manifest
        .ssr_module_mapping
        .entry(mod_id_str)
        .or_insert_with(HashMap::new);

      if let Some(client_module) = manifest.client_modules.get(export_name) {
        mapping.insert(
          "*".to_string(),
          ManifestEntry {
            id: module_info.module_id.clone(),
            name: client_module.name.clone(),
            // During SSR, we don't have external chunks to load on the server
            // side with our architecture of Webpack / Turbopack. We can keep
            // this field empty to save some bytes.
            chunks: vec![],
            r#async: module_info.r#async,
          },
        );
      }
    }

    if let Some(edge_module_info) = plugin_state
      .edge_ssr_modules
      .get(&final_ssr_named_module_id)
    {
      let mod_id_str = mod_id.to_string();
      let edge_mapping = manifest
        .edge_ssr_module_mapping
        .entry(mod_id_str)
        .or_insert_with(HashMap::new);

      if let Some(client_module) = manifest.client_modules.get(export_name) {
        edge_mapping.insert(
          "*".to_string(),
          ManifestEntry {
            id: edge_module_info.module_id.clone(),
            name: client_module.name.clone(),
            // During SSR, we don't have external chunks to load on the server
            // side with our architecture of Webpack / Turbopack. We can keep
            // this field empty to save some bytes.
            chunks: vec![],
            r#async: edge_module_info.r#async,
          },
        );
      }
    }
  }

  // addRSCIdMapping
  {
    let export_name = &final_resource;

    if let Some(module_info) = plugin_state.rsc_modules.get(&rsc_named_module_id) {
      let mod_id_str = mod_id.to_string();
      let rsc_mapping = manifest
        .rsc_module_mapping
        .entry(mod_id_str)
        .or_insert_with(HashMap::new);

      if let Some(client_module) = manifest.client_modules.get(export_name) {
        rsc_mapping.insert(
          "*".to_string(),
          ManifestEntry {
            id: module_info.module_id.clone(),
            name: client_module.name.clone(),
            // During SSR, we don't have external chunks to load on the server
            // side with our architecture of Webpack / Turbopack. We can keep
            // this field empty to save some bytes.
            chunks: vec![],
            r#async: module_info.r#async,
          },
        );
      }
    }

    if let Some(edge_module_info) = plugin_state.ssr_modules.get(&rsc_named_module_id) {
      let mod_id_str = mod_id.to_string();
      let edge_rsc_mapping = manifest
        .edge_rsc_module_mapping
        .entry(mod_id_str)
        .or_insert_with(HashMap::new);

      if let Some(client_module) = manifest.client_modules.get(export_name) {
        edge_rsc_mapping.insert(
          "*".to_string(),
          ManifestEntry {
            id: edge_module_info.module_id.clone(),
            name: client_module.name.clone(),
            // During SSR, we don't have external chunks to load on the server
            // side with our architecture of Webpack / Turbopack. We can keep
            // this field empty to save some bytes.
            chunks: vec![],
            r#async: edge_module_info.r#async,
          },
        );
      }
    }
  }

  Ok(())
}

fn record_chunk_group(
  chunk_group_ukey: &ChunkGroupUkey,
  compilation: &Compilation,
  checked_chunk_groups: &mut FxHashSet<ChunkGroupUkey>,
  checked_chunks: &mut FxHashSet<ChunkUkey>,
) {
  // Ensure recursion is stopped if we've already checked this chunk group.
  if checked_chunk_groups.contains(&chunk_group_ukey) {
    return;
  }
  checked_chunk_groups.insert(*chunk_group_ukey);

  let Some(chunk_group) = compilation.chunk_group_by_ukey.get(chunk_group_ukey) else {
    return;
  };

  // Only apply following logic to client module requests from client entry,
  // or if the module is marked as client module. That's because other
  // client modules don't need to be in the manifest at all as they're
  // never be referenced by the server/client boundary.
  // This saves a lot of bytes in the manifest.
  for chunk_ukey in &chunk_group.chunks {
    // Ensure recursion is stopped if we've already checked this chunk.
    if checked_chunks.contains(chunk_ukey) {
      continue;
    }
    checked_chunks.insert(*chunk_ukey);

    let entry_mods = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);

    for module_identifier in entry_mods {
      let Some(module) = compilation.module_by_identifier(&module_identifier) else {
        continue;
      };
      let Some(normal_module) = module.as_normal_module() else {
        continue;
      };
      if !normal_module
        .get_layer()
        .is_some_and(|layer| layer.as_str() == "react-client-components")
      {
        continue;
      }

      let module_graph = compilation.get_module_graph();
      let connections = module_graph.get_ordered_outgoing_connections(&module_identifier);

      for connection in connections {
        if let Some(client_entry_mod_identifier) =
          module_graph.get_resolved_module(&connection.dependency_id)
        {
          let mod_id = ChunkGraph::get_module_id(
            &compilation.module_ids_artifact,
            *client_entry_mod_identifier,
          );

          if let Some(id) = mod_id {
            record_module(id, &client_entry_mod_identifier);
          } else {
            let client_entry_mod_identifier = connection.module_identifier();
            // If this is a concatenation, register each child to the parent ID.
            let Some(client_entry_mod) =
              compilation.module_by_identifier(client_entry_mod_identifier)
            else {
              continue;
            };
            if let Some(concatenated_module) = client_entry_mod.as_concatenated_module() {
              let mod_id = ChunkGraph::get_module_id(
                &compilation.module_ids_artifact,
                concatenated_module.identifier(),
              );
              if let Some(concatenated_mod_id) = mod_id {
                record_module(concatenated_mod_id, &client_entry_mod_identifier);
              }
            }
          }
        }
      }
    }
  }

  // Walk through all children chunk groups too.
  for child in chunk_group.children_iterable() {
    record_chunk_group(child, compilation, checked_chunk_groups, checked_chunks);
  }
}

#[plugin]
#[derive(Debug, Default)]
// TODO: 这个插件在 server compiler 和 client compiler 哪里调用都行
// next.js 是在 client compiler 里调用的，我们看情况
pub struct ClientReferenceManifestPlugin {}

impl ClientReferenceManifestPlugin {
  pub fn new() -> Self {
    Self {
      inner: Default::default(),
    }
  }

  async fn create_asset(&self, compilation: &Compilation) -> Result<()> {
    for (entry_name, entrypoint_ukey) in &compilation.entrypoints {
      let manifest = ClientReferenceManifest {
        client_modules: Default::default(),
        rsc_module_mapping: Default::default(),
        module_loading: ModuleLoading {
          prefix: "".to_string(),
          cross_origin: None,
        },
        ssr_module_mapping: Default::default(),
        entry_css_files: Default::default(),
        entry_js_files: Default::default(),
      };
    }

    Ok(())
  }
}

#[plugin_hook(CompilationProcessAssets for ClientReferenceManifestPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ClientReferenceManifestPlugin");

  let start = logger.time("create client reference manifest");
  self.create_asset(compilation).await?;
  logger.time_end(start);

  Ok(())
}

impl Plugin for ClientReferenceManifestPlugin {
  fn name(&self) -> &'static str {
    "rspack.ClientReferenceManifestPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

use std::collections::HashMap;
use std::time::Instant;

use indexmap::IndexMap;
use itertools::Itertools;
use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{AssetInfo, Compilation, CompilationAsset, ExportInfoProvided};
use rspack_error::Result;
use serde_json::to_string;

use super::server_action::generate_action_id;
use crate::utils::constants::RSC_SERVER_ACTION_ENTRY_RE;
use crate::utils::decl::{ServerActionRef, ServerActions, ServerRef, ServerReferenceManifest};
use crate::utils::shared_data::{SHARED_CLIENT_IMPORTS, SHARED_DATA, SHARED_SERVER_IMPORTS};
use crate::utils::{has_client_directive, has_server_directive, is_same_asset};

#[derive(Debug, Default, Clone)]
pub struct RSCServerReferenceManifest {}

impl RSCServerReferenceManifest {
  fn add_server_ref(
    &self,
    id: &str,
    name: &str,
    chunks: &Vec<&String>,
    ssr_module_mapping: &mut HashMap<String, HashMap<String, ServerRef>>,
  ) {
    let module_mapping = ssr_module_mapping
      .entry(id.into())
      .or_insert_with(HashMap::default);
    module_mapping.insert(
      name.into(),
      ServerRef {
        id: id.to_string(),
        name: name.to_string(),
        chunks: chunks.iter().map(|&chunk| chunk.to_string()).collect(),
      },
    );
  }
  fn add_server_import_ref(
    &self,
    resource: &str,
    names: Vec<String>,
    server_ref: &mut ServerReferenceManifest,
  ) {
    for name in &names {
      let action_id = generate_action_id(resource, &name);
      server_ref
        .server_actions
        .insert(action_id.to_string(), HashMap::default());
    }
    server_ref
      .server_imports
      .insert(resource.to_string(), ServerActionRef { names });
  }
  fn add_server_action_ref(
    &self,
    module_id: &Option<String>,
    chunk_group_name: &str,
    server_actions_ref: &mut HashMap<String, HashMap<String, String>>,
  ) {
    if let Some(module_id) = module_id {
      let mut server_action_module_mapping = HashMap::default();
      server_action_module_mapping.insert(String::from("server"), module_id.to_string());
      server_actions_ref.insert(chunk_group_name.to_string(), server_action_module_mapping);
    }
  }
  async fn is_client_request(&self, resource_path: &str) -> bool {
    let client_imports = SHARED_CLIENT_IMPORTS.read().await;
    return client_imports.values().any(|f| f.contains(resource_path));
  }
  async fn is_server_request(&self, resource_path: &str) -> bool {
    let server_imports = SHARED_SERVER_IMPORTS.read().await;
    return server_imports.values().any(|f| f.contains(resource_path));
  }
  pub async fn process_assets_stage_optimize_hash(
    &self,
    compilation: &mut Compilation,
  ) -> Result<()> {
    println!("guard");
    let now = Instant::now();
    let mut server_manifest = ServerReferenceManifest {
      // client components module map used in server bundler manifest
      ssr_module_mapping: HashMap::default(),
      server_actions: IndexMap::default(),
      server_imports: HashMap::default(),
    };
    let mut mapping = HashMap::default();
    let mg = compilation.get_module_graph();

    for chunk_group in compilation.chunk_group_by_ukey.values() {
      let chunks = chunk_group
        .chunks
        .clone()
        .into_iter()
        .filter_map(|chunk| {
          let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
          let name_or_id = chunk.id.as_ref().or(chunk.name.as_ref());
          name_or_id.clone()
        })
        .collect::<Vec<_>>();
      for chunk in &chunk_group.chunks {
        let chunk_modules = compilation.chunk_graph.get_chunk_modules(chunk, &mg);
        for module in chunk_modules {
          let module_id = compilation.chunk_graph.get_module_id(module.identifier());
          let resolved_data = module
            .as_normal_module()
            .and_then(|m| Some(m.resource_resolved_data()));

          if resolved_data.is_none() {
            continue;
          }
          let is_client_components = match module.build_info() {
            Some(build_info) => has_client_directive(&build_info.directives),
            None => false,
          };
          let is_server_action = match module.build_info() {
            Some(build_info) => has_server_directive(&build_info.directives),
            None => false,
          };
          let resource = &resolved_data
            .expect("TODO:")
            .resource_path
            .to_str()
            .expect("TODO:");

          if chunk_group.name().is_some() {
            if RSC_SERVER_ACTION_ENTRY_RE.is_match(resource) {
              self.add_server_action_ref(module_id, chunk_group.name().unwrap(), &mut mapping);
            }
          }
          if !self.is_client_request(&resource).await && !self.is_server_request(&resource).await {
            continue;
          }
          if let Some(module_id) = module_id {
            let exports_info = mg.get_exports_info(&module.identifier());
            let module_exported_keys = exports_info.get_ordered_exports().filter_map(|id| {
              let info = id.get_export_info(&mg);
              if let Some(provided) = info.provided {
                match provided {
                  ExportInfoProvided::True => Some(info.name.clone()),
                  _ => None,
                }
              } else {
                None
              }
            });
            let mut names: Vec<String> = vec![];
            for name in module_exported_keys {
              if let Some(name) = name {
                names.push(name.to_string());
              }
            }
            if is_client_components {
              self.add_server_ref(
                module_id,
                "*",
                &chunks,
                &mut server_manifest.ssr_module_mapping,
              );
              self.add_server_ref(
                module_id,
                "",
                &chunks,
                &mut server_manifest.ssr_module_mapping,
              );
              for name in names.iter() {
                self.add_server_ref(
                  module_id,
                  name.as_str(),
                  &chunks,
                  &mut server_manifest.ssr_module_mapping,
                );
              }
            }
            if is_server_action {
              self.add_server_import_ref(resource, names, &mut server_manifest);
            }
          };
        }
      }
    }
    server_manifest
      .server_actions
      .clone()
      .keys()
      .sorted()
      .for_each(|f| {
        server_manifest
          .server_actions
          .insert(f.to_string(), mapping.clone());
      });
    let mut shared_data_guard = SHARED_DATA.write().await;
    *shared_data_guard = server_manifest.clone();
    let mut shim_server_manifest: HashMap<String, ServerActions> = HashMap::default();
    shim_server_manifest.insert(
      String::from("serverActions"),
      server_manifest.server_actions,
    );
    let content = to_string(&shim_server_manifest);
    match content {
      Ok(content) => {
        if !is_same_asset("server-reference-manifest.json", &content).await {
          let asset = CompilationAsset {
            source: Some(RawSource::from(content).boxed()),
            info: AssetInfo {
              immutable: false,
              ..AssetInfo::default()
            },
          };
          let filename = String::from("server-reference-manifest.json");
          // TODO: outputPath should be configable
          compilation.assets_mut().insert(filename, asset);
        }
      }
      Err(_) => (),
    }
    tracing::debug!(
      "make server-reference-manifest took {} ms.",
      now.elapsed().as_millis()
    );
    Ok(())
  }
}

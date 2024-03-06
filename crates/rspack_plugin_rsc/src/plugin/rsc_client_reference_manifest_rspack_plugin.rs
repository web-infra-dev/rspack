use std::collections::HashMap;

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  AssetInfo, CompilationAsset, ExportInfoProvided, Plugin, PluginContext,
  PluginProcessAssetsOutput, ProcessAssetsArgs,
};
use serde::Serialize;
use serde_json::to_string;

#[derive(Debug, Default, Clone)]
pub struct RSCClientReferenceManifestRspackPlugin {}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientReferenceManifest {
  client_modules: HashMap<String, ClientRef>,
}

#[derive(Debug, Serialize)]
pub struct ClientRef {
  pub id: String,
  pub name: String,
  pub chunks: Vec<String>,
}

impl RSCClientReferenceManifestRspackPlugin {
  pub fn new() -> Self {
    Self {}
  }
  fn get_client_ref_module_key(&self, filepath: &str, name: &str) -> String {
    if name == "*" {
      String::from(filepath)
    } else {
      format!("{}#{}", filepath, name)
    }
  }
  fn add_client_ref(
    &self,
    filepath: &str,
    id: &str,
    name: &str,
    chunks: &Vec<&String>,
    client_modules: &mut HashMap<String, ClientRef>,
  ) {
    let key = self.get_client_ref_module_key(filepath, name);
    client_modules.insert(
      key,
      ClientRef {
        id: id.to_string(),
        name: name.to_string(),
        chunks: chunks.iter().map(|&chunk| chunk.to_string()).collect(),
      },
    );
  }
}

#[async_trait::async_trait]
impl Plugin for RSCClientReferenceManifestRspackPlugin {
  async fn process_assets_stage_optimize_hash(
    &self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    let compilation = args.compilation;
    let mut client_manifest = ClientReferenceManifest {
      client_modules: HashMap::default(),
    };
    let use_client = String::from("use client");

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
        let chunk_modules = compilation
          .chunk_graph
          .get_chunk_modules(chunk, &compilation.module_graph);
        for module in chunk_modules {
          let module_id = compilation.chunk_graph.get_module_id(module.identifier());
          let resolved_data = module
            .as_normal_module()
            .and_then(|m| Some(m.resource_resolved_data()));

          if resolved_data.is_none() {
            continue;
          }
          // Skip non client modules
          if let Some(build_info) = module.build_info()
            && !build_info.directives.contains(&use_client)
          {
            continue;
          }
          let resource = &resolved_data.unwrap().resource;
          if let Some(module_id) = module_id {
            let exports_info = compilation
              .module_graph
              .get_exports_info(&module.identifier());
            let module_exported_keys = exports_info.get_ordered_exports().filter_map(|id| {
              let info = id.get_export_info(&compilation.module_graph);
              if let Some(provided) = info.provided {
                match provided {
                  ExportInfoProvided::True => Some(info.name.clone()),
                  _ => None,
                }
              } else {
                None
              }
            });
            self.add_client_ref(
              resource,
              module_id,
              "*",
              &chunks,
              &mut client_manifest.client_modules,
            );
            self.add_client_ref(
              resource,
              module_id,
              "",
              &chunks,
              &mut client_manifest.client_modules,
            );
            for name in module_exported_keys {
              if let Some(name) = name {
                self.add_client_ref(
                  resource,
                  module_id,
                  name.as_str(),
                  &chunks,
                  &mut client_manifest.client_modules,
                );
              }
            }
          };
        }
      }
    }
    let content = to_string(&client_manifest);
    match content {
      Ok(content) => {
        compilation.assets_mut().insert(
          String::from("client-reference-manifest.json"),
          CompilationAsset {
            source: Some(RawSource::from(content).boxed()),
            info: AssetInfo {
              immutable: false,
              ..AssetInfo::default()
            },
          },
        );
      }
      Err(_) => (),
    }
    Ok(())
  }
}

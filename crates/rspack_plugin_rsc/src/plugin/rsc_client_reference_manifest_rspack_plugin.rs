use std::collections::HashMap;
use std::time::Instant;

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  AssetInfo, Compilation, CompilationAsset, CompilationProcessAssets, ExportInfoProvided, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde::Serialize;
use serde_json::to_string;

use crate::utils::has_client_directive;

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCClientReferenceManifestRspackPlugin;
#[derive(Debug, Default, Clone)]
pub struct RSCClientReferenceManifest;

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

#[plugin_hook(CompilationProcessAssets for RSCClientReferenceManifestRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let plugin = RSCClientReferenceManifest {};
  plugin.process_assets_stage_optimize_hash(compilation)
}

impl RSCClientReferenceManifest {
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
  fn process_assets_stage_optimize_hash(&self, compilation: &mut Compilation) -> Result<()> {
    let now = Instant::now();
    let mut client_manifest = ClientReferenceManifest {
      client_modules: HashMap::default(),
    };
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
          // Skip non client modules
          if let Some(build_info) = module.build_info()
            && !has_client_directive(&build_info.directives)
          {
            continue;
          }
          let resource = &resolved_data.unwrap().resource;
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
    tracing::debug!(
      "make client-reference-manifest took {} ms.",
      now.elapsed().as_millis()
    );
    Ok(())
  }
}

#[async_trait::async_trait]
impl Plugin for RSCClientReferenceManifestRspackPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

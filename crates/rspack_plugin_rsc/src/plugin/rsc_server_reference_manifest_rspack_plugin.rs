use std::collections::HashMap;
use std::time::Instant;

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::{
  AssetInfo, Compilation, CompilationAsset, CompilationProcessAssets, ExportInfoProvided, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use serde_json::to_string;

use crate::utils::has_client_directive;
use crate::utils::reference_manifest::{ServerRef, ServerReferenceManifest};
use crate::utils::shared_data::{SHARED_CLIENT_IMPORTS, SHARED_DATA};

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCServerReferenceManifestRspackPlugin;
#[derive(Debug, Default, Clone)]
pub struct RSCServerReferenceManifest;

#[plugin_hook(CompilationProcessAssets for RSCServerReferenceManifestRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let plugin = RSCServerReferenceManifest {};
  plugin.process_assets_stage_optimize_hash(compilation)
}

impl RSCServerReferenceManifest {
  fn add_server_ref(
    &self,
    id: &str,
    name: &str,
    chunks: &Vec<&String>,
    ssr_module_mapping: &mut HashMap<String, HashMap<String, ServerRef>>,
  ) {
    if ssr_module_mapping.get(id).is_none() {
      ssr_module_mapping.insert(id.to_string(), HashMap::default());
    }
    let module_mapping = ssr_module_mapping.get_mut(id);
    match module_mapping {
      Some(mm) => {
        mm.insert(
          name.to_string(),
          ServerRef {
            id: id.to_string(),
            name: name.to_string(),
            chunks: chunks.iter().map(|&chunk| chunk.to_string()).collect(),
          },
        );
      }
      None => (),
    }
  }
  fn is_client_request(&self, resource_path: &str) -> bool {
    let client_imports = SHARED_CLIENT_IMPORTS.get();
    if let Some(client_imports) = client_imports {
      client_imports.values().any(|f| f.contains(resource_path))
    } else {
      true
    }
  }
  fn process_assets_stage_optimize_hash(&self, compilation: &mut Compilation) -> Result<()> {
    let now = Instant::now();
    let mut server_manifest = ServerReferenceManifest {
      ssr_module_mapping: HashMap::default(),
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
          let resource = &resolved_data
            .expect("TODO:")
            .resource_path
            .to_str()
            .expect("TODO:");
          if !self.is_client_request(&resource) {
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
            for name in module_exported_keys {
              if let Some(name) = name {
                self.add_server_ref(
                  module_id,
                  name.as_str(),
                  &chunks,
                  &mut server_manifest.ssr_module_mapping,
                );
              }
            }
          };
        }
      }
    }
    let _ = SHARED_DATA.set(server_manifest.clone());
    let content = to_string(&server_manifest);
    match content {
      Ok(content) => {
        // TODO: outputPath should be configable
        compilation.emit_asset(
          String::from("server-reference-manifest.json"),
          CompilationAsset {
            source: Some(RawSource::from(content).boxed()),
            info: AssetInfo {
              immutable: false,
              ..AssetInfo::default()
            },
          },
        )
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

// TODO: merge with rsc client entry rspack plugin
#[async_trait::async_trait]
impl Plugin for RSCServerReferenceManifestRspackPlugin {
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

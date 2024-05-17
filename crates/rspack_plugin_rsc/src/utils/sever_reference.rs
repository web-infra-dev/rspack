use std::collections::HashMap;
use std::time::Instant;

use rspack_core::{Compilation, ExportInfoProvided};
use rspack_error::Result;
use rspack_hook::plugin;

use crate::utils::decl::{ServerRef, ServerReferenceManifest};
use crate::utils::has_client_directive;
use crate::utils::shared_data::{SHARED_CLIENT_IMPORTS, SHARED_DATA};

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCServerReferenceManifestRspackPlugin;
#[derive(Debug, Default, Clone)]
pub struct RSCServerReferenceManifest;

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
    let client_imports = SHARED_CLIENT_IMPORTS.lock().unwrap();
    return client_imports.values().any(|f| f.contains(resource_path));
  }
  pub fn process_assets_stage_optimize_hash(&self, compilation: &mut Compilation) -> Result<()> {
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
    *SHARED_DATA.lock().unwrap() = server_manifest.clone();
    tracing::debug!(
      "make client-reference-manifest took {} ms.",
      now.elapsed().as_millis()
    );
    Ok(())
  }
}

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::EntryOptions;
use rspack_core::{
  AssetInfo, Compilation, CompilationAsset, CompilationProcessAssets, CompilerFinishMake,
  EntryDependency, ExportInfoProvided, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::path::relative;
use serde_json::to_string;
use sugar_path::SugarPath;

use crate::utils::decl::{
  ClientRef, ClientReferenceManifest, ReactRoute, ServerRef, ServerReferenceManifest,
};
use crate::utils::file::is_same_asset;
use crate::utils::has_client_directive;
use crate::utils::shared_data::{SHARED_CLIENT_IMPORTS, SHARED_DATA};

#[derive(Debug, Default, Clone)]
pub struct RSCClientReferenceManifestRspackPluginOptions {
  pub routes: Vec<ReactRoute>,
}

#[plugin]
#[derive(Debug, Default, Clone)]
pub struct RSCClientReferenceManifestRspackPlugin {
  pub options: RSCClientReferenceManifestRspackPluginOptions,
}

impl RSCClientReferenceManifestRspackPlugin {
  pub fn new(options: RSCClientReferenceManifestRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }
  async fn add_entry(&self, compilation: &mut Compilation) -> Result<()> {
    // TODO: server-entry is Server compiler entry chunk name
    // we should read it from SHARED_CLIENT_IMPORTS, in this way we do not need options.routes config
    // however, access SHARED_CLIENT_IMPORTS will throw thread error
    let context = compilation.options.context.clone();
    let request = format!(
      "rsc-client-entry-loader.js?from={}&name={}",
      "client-entry", "server-entry"
    );
    let entry = Box::new(EntryDependency::new(request, context.clone(), false));
    compilation
      .add_include(
        entry,
        EntryOptions {
          name: Some(String::from("client-entry")),
          ..Default::default()
        },
      )
      .await?;
    for ReactRoute { name, .. } in self.options.routes.clone() {
      let request = format!(
        "rsc-client-entry-loader.js?from={}&name={}",
        "route-entry", name
      );
      let entry = Box::new(EntryDependency::new(request, context.clone(), false));
      compilation
        .add_include(
          entry,
          EntryOptions {
            name: Some(String::from("client-entry")),
            ..Default::default()
          },
        )
        .await?;
    }
    Ok(())
  }
}

#[derive(Debug, Default, Clone)]
pub struct RSCClientReferenceManifest;

#[plugin_hook(CompilationProcessAssets for RSCClientReferenceManifestRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_HASH)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let plugin = RSCClientReferenceManifest {};
  plugin.process_assets_stage_optimize_hash(compilation)
}

#[plugin_hook(CompilerFinishMake for RSCClientReferenceManifestRspackPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  self.add_entry(compilation).await?;
  Ok(())
}

impl RSCClientReferenceManifest {
  fn normalize_module_id(&self, module_path: &PathBuf) -> String {
    let path_str = module_path.to_str().expect("TODO:");
    if !path_str.starts_with(".") {
      format!("./{}", path_str)
    } else {
      String::from(path_str)
    }
  }
  fn get_client_ref_module_key(&self, filepath: &str, name: &str) -> String {
    if name == "*" {
      String::from(filepath)
    } else {
      format!("{}#{}", filepath, name)
    }
  }
  fn is_client_request(&self, resource_path: &str) -> bool {
    let client_imports = SHARED_CLIENT_IMPORTS.lock().unwrap();
    return client_imports.values().any(|f| f.contains(resource_path));
  }
  fn add_server_ref(
    &self,
    module_id: &str,
    ssr_module_id: &str,
    name: &str,
    shared_ssr_module_mapping: &HashMap<String, HashMap<String, ServerRef>>,
    ssr_module_mapping: &mut HashMap<String, HashMap<String, ServerRef>>,
  ) {
    if ssr_module_mapping.get(module_id).is_none() {
      ssr_module_mapping.insert(module_id.to_string(), HashMap::default());
    }
    let module_mapping = ssr_module_mapping.get_mut(module_id).unwrap();
    let shared_module_mapping = shared_ssr_module_mapping.get(ssr_module_id);
    match shared_module_mapping {
      Some(smm) => {
        let server_ref = smm.get(name);
        match server_ref {
          Some(server_ref) => {
            module_mapping.insert(name.to_string(), server_ref.clone());
          }
          None => (),
        }
      }
      None => (),
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
    let mut client_manifest = ClientReferenceManifest::default();
    let shared_server_manifest = SHARED_DATA.lock().unwrap();
    let mut server_manifest = ServerReferenceManifest::default();
    let mg = compilation.get_module_graph();
    let context = &compilation.options.context;

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
            let ssr_module_path = relative(context.as_path(), resource.as_path());
            let ssr_module_id = self.normalize_module_id(&ssr_module_path);
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
            self.add_server_ref(
              module_id,
              ssr_module_id.as_str(),
              "*",
              &shared_server_manifest.ssr_module_mapping,
              &mut server_manifest.ssr_module_mapping,
            );
            self.add_server_ref(
              module_id,
              ssr_module_id.as_str(),
              "",
              &shared_server_manifest.ssr_module_mapping,
              &mut server_manifest.ssr_module_mapping,
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
                self.add_server_ref(
                  module_id,
                  ssr_module_id.as_str(),
                  name.as_str(),
                  &shared_server_manifest.ssr_module_mapping,
                  &mut server_manifest.ssr_module_mapping,
                );
              }
            }
          };
        }
      }
    }
    client_manifest.ssr_module_mapping = server_manifest.ssr_module_mapping;
    let content = to_string(&client_manifest);
    match content {
      Ok(content) => {
        // TODO: outputPath should be configable
        if !is_same_asset("client-reference-manifest.json", &content) {
          compilation.emit_asset(
            String::from("../server/client-reference-manifest.json"),
            CompilationAsset {
              source: Some(RawSource::from(content).boxed()),
              info: AssetInfo {
                immutable: false,
                ..AssetInfo::default()
              },
            },
          )
        }
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
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

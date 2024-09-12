use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use rspack_core::rspack_sources::{RawSource, SourceExt};
use rspack_core::EntryOptions;
use rspack_core::{
  AssetInfo, Compilation, CompilationAsset, CompilationProcessAssets, CompilerMake,
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
use crate::utils::file::generate_asset_version;
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
    let entry = Box::new(EntryDependency::new(request, context.clone(), None, false));
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
      let entry = Box::new(EntryDependency::new(request, context.clone(), None, false));
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
  plugin.process_assets_stage_optimize_hash(compilation).await
}

#[plugin_hook(CompilerMake for RSCClientReferenceManifestRspackPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
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
  async fn is_client_request(&self, resource_path: &str) -> bool {
    let client_imports = SHARED_CLIENT_IMPORTS.read().await;
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
    let module_mapping = ssr_module_mapping
      .entry(module_id.into())
      .or_insert_with(HashMap::default);
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
  async fn process_assets_stage_optimize_hash(&self, compilation: &mut Compilation) -> Result<()> {
    let now = Instant::now();
    let mut client_manifest = ClientReferenceManifest::default();
    let shared_server_manifest = SHARED_DATA.read().await;
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
          let request = module
            .as_normal_module()
            .and_then(|f| Some(f.user_request()));
          let module_id = compilation.chunk_graph.get_module_id(module.identifier());
          // FIXME: module maybe not a normal module, e.g. concatedmodule, not contain user_request
          // use name_for_condition as fallback
          // should be care user_request has resource query but name_for_condition not contain resource_query
          let name_for_condition = module.name_for_condition();

          let resource_path = request.or_else(|| name_for_condition.as_deref());
          if resource_path.is_none() {
            continue;
          }

          if module.build_info().is_none() {
            continue;
          }
          let resource = resource_path.unwrap();
          let is_client = self.is_client_request(&resource).await;

          if !is_client {
            continue;
          }
          if let Some(module_id) = module_id {
            let exports_info = mg.get_exports_info(&module.identifier());
            let module_exported_keys = exports_info.ordered_exports(&mg).filter_map(|id| {
              let provided = id.provided(&mg);
              let name = id.name(&mg);
              if let Some(provided) = provided {
                match provided {
                  ExportInfoProvided::True => Some(name.clone()),
                  _ => None,
                }
              } else {
                None
              }
            });
            let ssr_module_path = relative(context.as_ref(), resource.as_path());
            let ssr_module_id = self.normalize_module_id(&ssr_module_path);
            self.add_client_ref(
              &resource,
              module_id,
              "*",
              &chunks,
              &mut client_manifest.client_modules,
            );
            self.add_client_ref(
              &resource,
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
                  &resource,
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
        compilation.emit_asset(
          String::from("../server/client-reference-manifest.json"),
          CompilationAsset {
            source: Some(RawSource::from(content.as_str()).boxed()),
            info: AssetInfo {
              immutable: false,
              version: generate_asset_version(&content),
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

#[async_trait::async_trait]
impl Plugin for RSCClientReferenceManifestRspackPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx.context.compiler_hooks.make.tap(make::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

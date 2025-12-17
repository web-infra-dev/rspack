use std::sync::{Arc, LazyLock};

use derive_more::Debug;
use regex::Regex;
use rspack_collections::{Identifiable, IdentifierSet};
use rspack_core::{
  AssetInfo, AsyncDependenciesBlock, BoxDependency, ChunkGraph, ChunkGroup, ChunkGroupUkey,
  ChunkUkey, ClientEntryType, Compilation, CompilationAsset, CompilationProcessAssets,
  CompilerFinishMake, CompilerId, CrossOriginLoading, Dependency, DependencyId, EntryDependency,
  EntryOptions, ExportsInfoGetter, GroupOptions, Logger, Module, ModuleGraph, ModuleId,
  ModuleIdentifier, ModuleType, NormalModule, Plugin, PrefetchExportsInfoMode, RSCMeta,
  RSCModuleType, RuntimeSpec,
  build_module_graph::{UpdateParam, update_module_graph},
  rspack_sources::{RawStringSource, SourceExt, SourceValue},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, ESMExportImportedSpecifierDependency,
  ESMImportSpecifierDependency,
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sugar_path::SugarPath;
use swc_core::{atoms::Wtf8Atom, common::plugin};

use crate::{
  client_reference_dependency::ClientReferenceDependency,
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  reference_manifest::{ClientReferenceManifest, CrossOriginMode, ManifestExport, ModuleLoading},
  utils::{EntryModules, GetServerCompilerId},
};

#[plugin]
#[derive(Debug, Default)]
// Plugin for Server Compiler that generates Client Reference and SSR Module Manifests.
pub struct ClientReferenceManifestPlugin {}

impl ClientReferenceManifestPlugin {
  pub fn new() -> Self {
    Self {
      inner: Default::default(),
    }
  }

  async fn create_asset(
    &self,
    compilation: &mut Compilation,
    plugin_state: &mut PluginState,
  ) -> Result<()> {
    let Some(module_loading) = plugin_state.module_loading.take() else {
      return Err(rspack_error::error!(
        "Module loading configuration is missing in plugin state. Ensure that the ReactClientPlugin is applied and configured correctly."
      ));
    };

    let mut manifest = ClientReferenceManifest {
      client_modules: Default::default(),
      module_loading,
      ssr_module_map: Default::default(),
      entry_css_files: plugin_state.entry_css_files.clone(),
      entry_js_files: Default::default(),
    };

    for (resource, client_manifest_export) in plugin_state.client_modules.drain() {
      if let Some(ssr_manifest_export) = plugin_state.ssr_modules.remove(&resource) {
        let mut v = FxHashMap::default();
        v.insert("*".to_string(), ssr_manifest_export);
        manifest
          .ssr_module_map
          .insert(client_manifest_export.id.to_string(), v);
      }

      manifest
        .client_modules
        .insert(resource, client_manifest_export);
    }

    let json = serde_json::to_string(&manifest).to_rspack_result()?;
    let source = RawStringSource::from(json.clone()).boxed();

    let assets = compilation.assets_mut();
    for asset in assets.values_mut() {
      if let Some(source) = asset.source.as_ref() {
        if let SourceValue::String(code) = source.source() {
          if code.contains("__RSPACK_RSC_CLIENT_REFERENCE_MANIFEST__") {
            asset.set_source(Some(
              RawStringSource::from(code.replace(
                "__RSPACK_RSC_CLIENT_REFERENCE_MANIFEST__",
                &format!(
                  "JSON.parse({})",
                  serde_json::to_string(&json).to_rspack_result()?
                ),
              ))
              .boxed(),
            ));
          }
        }
      }
    }

    compilation.emit_asset(
      "client-reference-manifest.json".into(),
      CompilationAsset::new(Some(source), AssetInfo::default()),
    );
    Ok(())
  }
}

#[plugin_hook(CompilationProcessAssets for ClientReferenceManifestPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.ClientReferenceManifestPlugin");

  let server_compiler_id = compilation.compiler_id();

  let mut guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let Some(plugin_state) = guard.get_mut(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let start = logger.time("create client reference manifest");
  self.create_asset(compilation, plugin_state).await?;
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

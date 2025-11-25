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
  rspack_sources::{RawStringSource, SourceExt},
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
  client_reference_manifest::{
    ClientReferenceManifest, CrossOriginMode, ManifestExport, ModuleLoading,
  },
  constants::WEBPACK_LAYERS,
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  utils::{EntryModules, GetServerCompilerId, ServerEntries},
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
    let mut manifest = ClientReferenceManifest {
      client_modules: Default::default(),
      module_loading: ModuleLoading {
        prefix: "".to_string(),
        cross_origin: None,
      },
      ssr_module_mapping: Default::default(),
      entry_css_files: Default::default(),
      entry_js_files: Default::default(),
    };

    for (resource_id, manifest_export) in plugin_state.client_modules.drain() {
      manifest
        .client_modules
        .insert(resource_id.clone(), manifest_export);
    }

    let json = serde_json::to_string_pretty(&manifest).to_rspack_result()?;
    let source = RawStringSource::from(json).boxed();
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

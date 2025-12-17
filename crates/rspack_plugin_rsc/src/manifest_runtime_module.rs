use indoc::formatdoc;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkGraph, ChunkUkey, Compilation, ModuleGraph, RuntimeModule, RuntimeModuleStage,
  impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rustc_hash::FxHashMap;
use serde::Serialize;

use crate::{
  loaders::action_entry_loader::parse_action_entries,
  plugin_state::{PLUGIN_STATE_BY_COMPILER_ID, PluginState},
  reference_manifest::{ManifestExport, ManifestNode},
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RscManifestRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl RscManifestRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/rsc_manifest"), None)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RscManifestRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let server_compiler_id = compilation.compiler_id();

    let mut state_by_compiler_id = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
    let plugin_state = state_by_compiler_id
      .get_mut(&server_compiler_id)
      .ok_or_else(|| {
        rspack_error::error!(
          "Failed to find RSC plugin state for compiler (ID: {}).",
          server_compiler_id.as_u32()
        )
      })?;
    let server_manifest = build_server_manifest(compilation, plugin_state)?;
    let server_manifest_literal = to_json_string_literal(server_manifest).to_rspack_result()?;

    let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
      rspack_error::error!(
        "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
      )
    })?;

    let mut ssr_module_map: FxHashMap<String, ManifestNode> = Default::default();
    for (resource, client_manifest_export) in &plugin_state.client_modules {
      if let Some(ssr_manifest_export) = plugin_state.ssr_modules.get(resource) {
        let mut v = FxHashMap::default();
        v.insert("*".to_string(), ssr_manifest_export.clone());
        ssr_module_map.insert(client_manifest_export.id.to_string(), v);
      }
    }

    let client_manifest_literal =
      to_json_string_literal(&plugin_state.client_modules).to_rspack_result()?;
    let ssr_module_map_literal = to_json_string_literal(&ssr_module_map).to_rspack_result()?;
    let module_loading_literal = to_json_string_literal(module_loading).to_rspack_result()?;
    let entry_css_files_literal =
      to_json_string_literal(&plugin_state.entry_css_files).to_rspack_result()?;
    let entry_js_files_literal =
      to_json_string_literal(&plugin_state.entry_js_files).to_rspack_result()?;

    Ok(formatdoc! {
        r#"
          __webpack_require__.rscM = {{}};
          __webpack_require__.rscM.serverManifest = JSON.parse({server_manifest});
          __webpack_require__.rscM.clientManifest = JSON.parse({client_manifest});
          __webpack_require__.rscM.ssrModuleMap = JSON.parse({ssr_module_map});
          __webpack_require__.rscM.moduleLoading = JSON.parse({module_loading});
          __webpack_require__.rscM.entryCssFiles = JSON.parse({entry_css_files});
          __webpack_require__.rscM.entryJsFiles = JSON.parse({entry_js_files});
        "#,
      server_manifest = server_manifest_literal,
      client_manifest = client_manifest_literal,
      ssr_module_map = ssr_module_map_literal,
      module_loading = module_loading_literal,
      entry_css_files = entry_css_files_literal,
      entry_js_files = entry_js_files_literal,
    })
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

/// Returns a JSON string literal for `value` (i.e. double-encoded), suitable for embedding into JS.
///
/// Example:
/// - input:  `{"a":1}`
/// - output: "\"{\\\"a\\\":1}\""
fn to_json_string_literal<T: ?Sized + Serialize>(value: &T) -> Result<String> {
  serde_json::to_string(&serde_json::to_string(value).to_rspack_result()?).to_rspack_result()
}

fn build_server_manifest<'a>(
  compilation: &Compilation,
  plugin_state: &'a mut PluginState,
) -> Result<&'a FxHashMap<String, ManifestExport>> {
  let server_actions = &mut plugin_state.server_actions;

  // traverse modules
  for chunk_group in compilation.chunk_group_by_ukey.values() {
    for chunk_ukey in &chunk_group.chunks {
      let chunk_modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier(chunk_ukey);
      for module_identifier in chunk_modules {
        // Go through all action entries and record the module ID for each entry.
        let module = compilation.module_by_identifier(module_identifier);
        let Some(module) = module else {
          continue;
        };
        let Some(module) = module.as_normal_module() else {
          continue;
        };
        let request = module.request();
        let Some(module_id) =
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
        else {
          continue;
        };

        if request.starts_with("builtin:action-entry-loader") {
          let loader_query = request
            .splitn(2, '?')
            .nth(1)
            .unwrap_or_default()
            .rsplitn(2, '!')
            .nth(1)
            .unwrap_or_default();
          let loader_options = form_urlencoded::parse(loader_query.as_bytes());
          let mut individual_actions = vec![];
          for (k, v) in loader_options {
            if k == "actions" {
              individual_actions = parse_action_entries(v.as_ref())?.unwrap_or_default();
            }
          }
          for action in individual_actions {
            server_actions.insert(
              action.id.to_string(),
              ManifestExport {
                id: module_id.to_string(),
                name: action.id.to_string(),
                chunks: vec![],
                r#async: Some(ModuleGraph::is_async(&compilation, module_identifier)),
              },
            );
          }
        }
      }
    }
  }

  Ok(server_actions)
}

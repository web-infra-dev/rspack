#![allow(clippy::ref_option_ref)]

use indoc::formatdoc;
use rspack_core::{
  ChunkGraph, Compilation, Module, ModuleGraph, ModuleId, ModuleIdentifier, RuntimeGlobals,
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::{
  constants::LAYERS_NAMES,
  loaders::action_entry_loader::{ACTION_ENTRY_LOADER_IDENTIFIER, parse_action_entries},
  plugin_state::PLUGIN_STATES,
  reference_manifest::{ManifestExport, ManifestNode, ModuleLoading, ServerReferenceManifest},
  utils::{ChunkModules, get_module_resource, to_json_string_literal},
};

fn serialize_none_as_empty_object<S, T>(val: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  T: Serialize,
{
  match val {
    Some(v) => v.serialize(serializer),
    None => {
      let map = serializer.serialize_map(Some(0))?;
      map.end()
    }
  }
}

#[allow(clippy::ref_option_ref)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RscManifest<'a> {
  pub server_manifest: &'a FxHashMap<String, ManifestExport>,
  pub client_manifest: &'a FxHashMap<String, ManifestExport>,
  pub server_consumer_module_map: &'a FxHashMap<String, ManifestNode>,
  pub module_loading: &'a ModuleLoading,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_css_files: Option<&'a FxHashMap<String, FxIndexSet<String>>>,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_js_files: Option<&'a FxIndexSet<String>>,
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct RscManifestRuntimeModule {}

impl RscManifestRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RscManifestRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let server_compiler_id = compilation.compiler_id();

    let Some(entry_name) = self
      .chunk
      .as_ref()
      .and_then(|chunk_ukey| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .get(chunk_ukey)
      })
      .and_then(|chunk| {
        chunk.get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      })
      .and_then(|entry_options| entry_options.name.as_ref())
    else {
      return Ok(String::new());
    };

    let mut plugin_state = PLUGIN_STATES.get_mut(&server_compiler_id).ok_or_else(|| {
      rspack_error::error!(
        "Failed to find RSC plugin state for compiler (ID: {}).",
        server_compiler_id.as_u32()
      )
    })?;

    build_server_manifest(compilation, &mut plugin_state.server_actions)?;
    let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
      rspack_error::error!(
        "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
      )
    })?;
    let server_consumer_module_map =
      build_server_consumer_module_map(compilation, &plugin_state.client_modules);

    let rsc_manifest = RscManifest {
      server_manifest: &plugin_state.server_actions,
      client_manifest: &plugin_state.client_modules,
      server_consumer_module_map: &server_consumer_module_map,
      module_loading,
      entry_css_files: plugin_state.entry_css_files.get(entry_name),
      entry_js_files: plugin_state.entry_js_files.get(entry_name),
    };

    Ok(formatdoc! {
      r#"
        {require_name}.rscM = JSON.parse({rsc_manifest_json});
      "#,
      require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
      rsc_manifest_json = to_json_string_literal(&rsc_manifest).to_rspack_result()?,
    })
  }
}

fn build_server_manifest(
  compilation: &Compilation,
  server_actions: &mut ServerReferenceManifest,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let mut record_module =
    |module_identifier: &ModuleIdentifier, module_id: &ModuleId| -> Result<()> {
      let Some(module) = module_graph.module_by_identifier(module_identifier) else {
        return Ok(());
      };
      let Some(normal_module) = module.as_normal_module() else {
        return Ok(());
      };

      let request = normal_module.request();
      if !request.starts_with(ACTION_ENTRY_LOADER_IDENTIFIER) {
        return Ok(());
      }

      let loader_query = request
        .split_once('?')
        .map(|x| x.1)
        .unwrap_or_default()
        .rsplit_once('!')
        .map(|x| x.0)
        .unwrap_or_default();

      let loader_options = form_urlencoded::parse(loader_query.as_bytes());
      for (k, v) in loader_options {
        if k != "actions" {
          continue;
        }

        if let Some(actions) = parse_action_entries(v.into_owned())? {
          for action in actions {
            server_actions.insert(
              action.id.clone(),
              ManifestExport {
                id: module_id.to_string(),
                name: action.id.clone(),
                // Server Action modules serve as endpoints rather than code splitting points,
                // so ensuring chunk loading at runtime is unnecessary.
                chunks: vec![],
                r#async: Some(ModuleGraph::is_async(
                  &compilation.async_modules_artifact,
                  &module.identifier(),
                )),
              },
            );
          }
        }
        break;
      }

      Ok(())
    };

  for (_, module) in module_graph.modules() {
    let module_id =
      match ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier()) {
        Some(id) => id,
        None => continue,
      };

    if let Some(concatenated_module) = module.as_concatenated_module() {
      for inner_module in concatenated_module.get_modules() {
        record_module(&inner_module.id, module_id)?;
      }
      continue;
    }

    record_module(&module.identifier(), module_id)?;
  }

  Ok(())
}

fn build_server_consumer_module_map(
  compilation: &Compilation,
  client_modules: &FxHashMap<String, ManifestExport>,
) -> FxHashMap<String, ManifestNode> {
  let mut server_consumer_module_map: FxHashMap<String, ManifestNode> = Default::default();
  let module_graph = compilation.get_module_graph();
  let chunk_modules = ChunkModules::new(compilation, module_graph);

  let mut record_module = |module_identifier: &ModuleIdentifier, module_id: &ModuleId| {
    let Some(module) = module_graph.module_by_identifier(module_identifier) else {
      return;
    };
    let Some(normal_module) = module.as_normal_module() else {
      return;
    };

    if normal_module
      .get_layer()
      .is_none_or(|layer| layer != LAYERS_NAMES.server_side_rendering)
    {
      return;
    }

    let resource = get_module_resource(module.as_ref());
    if resource.is_empty() {
      return;
    }

    let manifest_export = ManifestExport {
      id: module_id.to_string(),
      name: "*".to_string(),
      chunks: vec![],
      r#async: Some(ModuleGraph::is_async(
        &compilation.async_modules_artifact,
        &module.identifier(),
      )),
    };

    let mut node = FxHashMap::default();
    node.insert("*".to_string(), manifest_export);

    if let Some(export) = client_modules.get(resource.as_ref()) {
      server_consumer_module_map.insert(export.id.clone(), node);
    }
  };

  for (module_identifier, module_id) in chunk_modules {
    let Some(module) = module_graph.module_by_identifier(&module_identifier) else {
      continue;
    };

    if let Some(concatenated_module) = module.as_concatenated_module() {
      for inner_module in concatenated_module.get_modules() {
        record_module(&inner_module.id, &module_id);
      }
    } else {
      record_module(&module_identifier, &module_id);
    }
  }
  server_consumer_module_map
}

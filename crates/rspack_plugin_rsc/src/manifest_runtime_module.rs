#![allow(clippy::ref_option_ref)]

use indoc::formatdoc;
use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::{
  plugin_state::PLUGIN_STATES,
  reference_manifest::{ManifestExport, ManifestNode, ModuleLoading},
  utils::to_json_string_literal,
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
struct RscManifestPerEntry<'a> {
  pub server_manifest: &'a FxHashMap<String, ManifestExport>,
  pub client_manifest: &'a FxHashMap<String, ManifestExport>,
  pub server_consumer_module_map: &'a FxHashMap<String, ManifestNode>,
  pub module_loading: &'a ModuleLoading,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_css_files: Option<&'a FxHashMap<String, FxIndexSet<String>>>,

  #[serde(serialize_with = "serialize_none_as_empty_object")]
  pub entry_js_files: Option<&'a FxIndexSet<String>>,
}

/// Full manifest (all entries) for the onManifest callback.
#[allow(clippy::ref_option_ref)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RscManifest<'a> {
  pub server_manifest: &'a FxHashMap<String, ManifestExport>,
  pub client_manifest: &'a FxHashMap<String, ManifestExport>,
  pub server_consumer_module_map: &'a FxHashMap<String, ManifestNode>,
  pub module_loading: &'a ModuleLoading,
  pub entry_css_files: &'a FxHashMap<String, FxHashMap<String, FxIndexSet<String>>>,
  pub entry_js_files: &'a FxHashMap<String, FxIndexSet<String>>,
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

    let plugin_state = PLUGIN_STATES.get(&server_compiler_id).ok_or_else(|| {
      rspack_error::error!(
        "Failed to find RSC plugin state for compiler (ID: {}).",
        server_compiler_id.as_u32()
      )
    })?;

    let server_consumer_module_map = plugin_state.server_consumer_module_map.as_ref().ok_or_else(
      || {
        rspack_error::error!(
          "RSC server_consumer_module_map not found. Ensure process_assets hook ran for compiler (ID: {}).",
          server_compiler_id.as_u32()
        )
      },
    )?;
    let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
      rspack_error::error!(
        "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
      )
    })?;

    let rsc_manifest = RscManifestPerEntry {
      server_manifest: &plugin_state.server_actions,
      client_manifest: &plugin_state.client_modules,
      server_consumer_module_map,
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

use std::sync::Arc;

use indoc::formatdoc;
use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{
  plugin_state::PLUGIN_STATES, reference_manifest::RscEntryManifest, utils::to_json_string_literal,
};

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
      .and_then(|entry_options| entry_options.name.clone().map(Arc::from))
    else {
      return Ok(String::new());
    };

    let plugin_state = PLUGIN_STATES.get(&server_compiler_id).ok_or_else(|| {
      rspack_error::error!(
        "Failed to find RSC plugin state for compiler (ID: {}).",
        server_compiler_id.as_u32()
      )
    })?;

    let entry_state = plugin_state.entries.get(&entry_name).ok_or_else(|| {
      rspack_error::error!(
        "RSC entry state not found for entry {:?} (compiler ID: {}).",
        entry_name,
        server_compiler_id.as_u32()
      )
    })?;
    let server_manifest = &entry_state.server_actions;
    let client_manifest = &entry_state.client_modules;
    let server_consumer_module_map = entry_state.server_consumer_module_map.as_ref();
    let module_loading = plugin_state.module_loading.as_ref().ok_or_else(|| {
      rspack_error::error!(
        "Missing RSC moduleLoading config in plugin state. Ensure ClientPlugin is applied."
      )
    })?;

    let rsc_manifest = RscEntryManifest {
      server_manifest,
      client_manifest,
      server_consumer_module_map,
      module_loading,
      entry_css_files: &entry_state.entry_css_files,
      entry_js_files: &entry_state.entry_js_files,
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

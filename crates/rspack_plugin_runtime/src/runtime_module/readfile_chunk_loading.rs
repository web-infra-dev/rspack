use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  BooleanMatcher, Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  RuntimeTemplate, compile_boolean_matcher, impl_runtime_module,
};
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;

use super::{generate_javascript_hmr_runtime, utils::get_output_dir};
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ReadFileChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl ReadFileChunkLoadingRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}readfile_chunk_loading",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }
}

impl ReadFileChunkLoadingRuntimeModule {
  fn generate_base_uri(
    &self,
    chunk: &Chunk,
    compilation: &Compilation,
    root_output_dir: &str,
  ) -> String {
    let base_uri = chunk
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| {
        format!(
          "require(\"url\").pathToFileURL({})",
          if !root_output_dir.is_empty() {
            format!(
              "__dirname + {}",
              serde_json::to_string(&format!("/{root_output_dir}"))
                .expect("should able to be serde_json::to_string")
            )
          } else {
            "__filename".to_string()
          }
        )
      });
    format!(
      "{} = {};\n",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::BASE_URI),
      base_uri
    )
  }

  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.to_string();

    match id {
      TemplateId::Raw => base_id,
      TemplateId::WithOnChunkLoad => format!("{base_id}_with_on_chunk_load"),
      TemplateId::WithLoading => format!("{base_id}_with_loading"),
      TemplateId::WithExternalInstallChunk => format!("{base_id}_with_external_install_chunk"),
      TemplateId::WithHmr => format!("{base_id}_with_hmr"),
      TemplateId::WithHmrManifest => format!("{base_id}_with_hmr_manifest"),
      TemplateId::HmrRuntime => format!("{base_id}_hmr_runtime"),
    }
  }
}

#[allow(clippy::enum_variant_names)]
enum TemplateId {
  Raw,
  WithOnChunkLoad,
  WithLoading,
  WithExternalInstallChunk,
  WithHmr,
  WithHmrManifest,
  HmrRuntime,
}

#[async_trait::async_trait]
impl RuntimeModule for ReadFileChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::WithOnChunkLoad),
        include_str!("runtime/readfile_chunk_loading_with_on_chunk_load.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::Raw),
        include_str!("runtime/readfile_chunk_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithLoading),
        include_str!("runtime/readfile_chunk_loading_with_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithExternalInstallChunk),
        include_str!("runtime/readfile_chunk_loading_with_external_install_chunk.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmr),
        include_str!("runtime/readfile_chunk_loading_with_hmr.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmrManifest),
        include_str!("runtime/readfile_chunk_loading_with_hmr_manifest.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::HmrRuntime),
        include_str!("runtime/javascript_hot_module_replacement.ejs").to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    let with_external_install_chunk =
      runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);

    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let root_output_dir = get_output_dir(chunk, compilation, false).await?;
    let mut source = String::default();

    if with_base_uri {
      source.push_str(&self.generate_base_uri(chunk, compilation, &root_output_dir));
    }

    if with_hmr {
      let state_expression = format!(
        "{}_readFileVm",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX)
      );
      source.push_str(&format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 0)
      ));
    } else {
      source.push_str(&format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 0)
      ));
    }

    if with_on_chunk_load {
      let source_with_on_chunk_load = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithOnChunkLoad), None)?;

      source.push_str(&source_with_on_chunk_load);
    }

    if with_loading || with_external_install_chunk {
      let raw_source = compilation.runtime_template.render(
        &self.template_id(TemplateId::Raw),
        Some(serde_json::json!({
          "_with_on_chunk_loaded": match with_on_chunk_load {
            true => format!("{}();", compilation.runtime_template.render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED)),
            false => String::new(),
          }
        })),
      )?;

      source.push_str(&raw_source);
    }

    if with_loading {
      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 0;".to_string()
      } else {
        compilation.runtime_template.render(
          &self.template_id(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_js_matcher": &has_js_matcher.render("chunkId"),
            "_output_dir": &root_output_dir,
            "_match_fallback": if matches!(has_js_matcher, BooleanMatcher::Condition(true)) {
              ""
            } else {
              "else installedChunks[chunkId] = 0;\n"
            },
          })),
        )?
      };

      source.push_str(&format!(
        r#"
        // ReadFile + VM.run chunk loading for javascript"
        {}.readFileVm = function (chunkId, promises) {{
          {body}
        }};
        "#,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      ));
    }

    if with_external_install_chunk {
      let source_with_external_install_chunk = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithExternalInstallChunk),
        None,
      )?;

      source.push_str(&source_with_external_install_chunk);
    }

    if with_hmr {
      let source_with_hmr = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmr), None)?;
      source.push_str(&source_with_hmr);
      let hmr_runtime = generate_javascript_hmr_runtime(
        &self.template_id(TemplateId::HmrRuntime),
        "readFileVm",
        &compilation.runtime_template,
      )?;
      source.push_str(&hmr_runtime);
    }

    if with_hmr_manifest {
      let source_with_hmr_manifest = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithHmrManifest),
        Some(serde_json::json!({
          "_output_dir":  &root_output_dir
        })),
      )?;

      source.push_str(&source_with_hmr_manifest);
    }

    Ok(source)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk)
  }
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY | RuntimeGlobals::HAS_OWN_PROPERTY
  }
}

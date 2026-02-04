use std::sync::LazyLock;

use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  BooleanMatcher, Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
  RuntimeTemplate, compile_boolean_matcher, impl_runtime_module,
};
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;

use super::{generate_javascript_hmr_runtime, utils::get_output_dir};
use crate::{
  extract_runtime_globals_from_ejs, get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
};

static REQUIRE_CHUNK_LOADING_TEMPLATE: &str = include_str!("runtime/require_chunk_loading.ejs");
static REQUIRE_CHUNK_LOADING_WITH_LOADING_TEMPLATE: &str =
  include_str!("runtime/require_chunk_loading_with_loading.ejs");
static REQUIRE_CHUNK_LOADING_WITH_LOADING_MATCHER_TEMPLATE: &str =
  include_str!("runtime/require_chunk_loading_with_loading_matcher.ejs");
static REQUIRE_CHUNK_LOADING_WITH_ON_CHUNK_LOAD_TEMPLATE: &str =
  include_str!("runtime/require_chunk_loading_with_on_chunk_load.ejs");
static REQUIRE_CHUNK_LOADING_WITH_EXTERNAL_INSTALL_CHUNK_TEMPLATE: &str =
  include_str!("runtime/require_chunk_loading_with_external_install_chunk.ejs");
static REQUIRE_CHUNK_LOADING_WITH_HMR_TEMPLATE: &str =
  include_str!("runtime/require_chunk_loading_with_hmr.ejs");
static REQUIRE_CHUNK_LOADING_WITH_HMR_MANIFEST_TEMPLATE: &str =
  include_str!("runtime/require_chunk_loading_with_hmr_manifest.ejs");
static JAVASCRIPT_HOT_MODULE_REPLACEMENT_TEMPLATE: &str =
  include_str!("runtime/javascript_hot_module_replacement.ejs");

static REQUIRE_CHUNK_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_TEMPLATE));
static REQUIRE_CHUNK_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_WITH_LOADING_TEMPLATE));
static REQUIRE_CHUNK_LOADING_WITH_LOADING_MATCHER_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_WITH_LOADING_MATCHER_TEMPLATE)
  });
static REQUIRE_CHUNK_LOADING_WITH_ON_CHUNK_LOAD_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_WITH_ON_CHUNK_LOAD_TEMPLATE)
  });
static REQUIRE_CHUNK_LOADING_WITH_EXTERNAL_INSTALL_CHUNK_RUNTIME_REQUIREMENTS: LazyLock<
  RuntimeGlobals,
> = LazyLock::new(|| {
  extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_WITH_EXTERNAL_INSTALL_CHUNK_TEMPLATE)
});
static REQUIRE_CHUNK_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_WITH_HMR_TEMPLATE));
static REQUIRE_CHUNK_LOADING_WITH_HMR_MANIFEST_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(REQUIRE_CHUNK_LOADING_WITH_HMR_MANIFEST_TEMPLATE)
  });
static JAVASCRIPT_HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    let mut res = extract_runtime_globals_from_ejs(JAVASCRIPT_HOT_MODULE_REPLACEMENT_TEMPLATE);
    // ensure chunk handlers is optional
    res.remove(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    res
  });

#[impl_runtime_module]
#[derive(Debug)]
pub struct RequireChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl RequireChunkLoadingRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}require_chunk_loading",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }
  fn generate_base_uri(
    &self,
    chunk: &Chunk,
    compilation: &Compilation,
    root_output_dir: &str,
  ) -> String {
    let base_uri = chunk
      .get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| {
        format!(
          "require(\"url\").pathToFileURL({})",
          if root_output_dir != "./" {
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
      TemplateId::WithLoading => format!("{}_with_loading", &base_id),
      TemplateId::WithLoadingMatcher => format!("{}_with_loading_matcher", &base_id),
      TemplateId::WithOnChunkLoad => format!("{}_with_on_chunk_load", &base_id),
      TemplateId::WithExternalInstallChunk => format!("{}_with_external_install_chunk", &base_id),
      TemplateId::WithHmr => format!("{}_with_hmr", &base_id),
      TemplateId::WithHmrManifest => format!("{}_with_hmr_manifest", &base_id),
      TemplateId::Raw => base_id,
      TemplateId::HmrRuntime => format!("{}_hmr_runtime", &base_id),
    }
  }

  pub fn get_runtime_requirements_basic() -> RuntimeGlobals {
    *REQUIRE_CHUNK_LOADING_RUNTIME_REQUIREMENTS
  }

  pub fn get_runtime_requirements_with_loading() -> RuntimeGlobals {
    *REQUIRE_CHUNK_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS
      | *REQUIRE_CHUNK_LOADING_WITH_LOADING_MATCHER_RUNTIME_REQUIREMENTS
  }

  pub fn get_runtime_requirements_with_on_chunk_load() -> RuntimeGlobals {
    *REQUIRE_CHUNK_LOADING_WITH_ON_CHUNK_LOAD_RUNTIME_REQUIREMENTS
  }

  pub fn get_runtime_requirements_with_hmr() -> RuntimeGlobals {
    *REQUIRE_CHUNK_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS
      | *JAVASCRIPT_HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS
  }

  pub fn get_runtime_requirements_with_hmr_manifest() -> RuntimeGlobals {
    *REQUIRE_CHUNK_LOADING_WITH_HMR_MANIFEST_RUNTIME_REQUIREMENTS
  }

  pub fn get_runtime_requirements_with_external_install_chunk() -> RuntimeGlobals {
    *REQUIRE_CHUNK_LOADING_WITH_EXTERNAL_INSTALL_CHUNK_RUNTIME_REQUIREMENTS
  }
}

#[allow(clippy::enum_variant_names)]
enum TemplateId {
  Raw,
  WithLoading,
  WithLoadingMatcher,
  WithOnChunkLoad,
  WithExternalInstallChunk,
  WithHmr,
  WithHmrManifest,
  HmrRuntime,
}

#[async_trait::async_trait]
impl RuntimeModule for RequireChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        REQUIRE_CHUNK_LOADING_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::WithLoading),
        REQUIRE_CHUNK_LOADING_WITH_LOADING_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::WithLoadingMatcher),
        REQUIRE_CHUNK_LOADING_WITH_LOADING_MATCHER_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::WithOnChunkLoad),
        REQUIRE_CHUNK_LOADING_WITH_ON_CHUNK_LOAD_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::WithExternalInstallChunk),
        REQUIRE_CHUNK_LOADING_WITH_EXTERNAL_INSTALL_CHUNK_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmr),
        REQUIRE_CHUNK_LOADING_WITH_HMR_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmrManifest),
        REQUIRE_CHUNK_LOADING_WITH_HMR_MANIFEST_TEMPLATE.to_string(),
      ),
      (
        self.template_id(TemplateId::HmrRuntime),
        JAVASCRIPT_HOT_MODULE_REPLACEMENT_TEMPLATE.to_string(),
      ),
    ]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_external_install_chunk =
      runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);

    let condition_map = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let root_output_dir = get_output_dir(chunk, compilation, true).await?;

    let mut source = String::default();

    if with_base_uri {
      source.push_str(&self.generate_base_uri(chunk, compilation, &root_output_dir));
    }

    if with_hmr {
      let state_expression = format!(
        "{}_require",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX)
      );
      source.push_str(&format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 1)
      ));
    } else {
      source.push_str(&format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 1)
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
            true => format!("{}();", compilation
                .runtime_template
                .render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED)),
            false => String::new(),
          }
        })),
      )?;

      source.push_str(&raw_source);
    }

    if with_loading {
      if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        let source_with_loading = compilation
          .runtime_template
          .render(&self.template_id(TemplateId::WithLoading), None)?;

        source.push_str(&source_with_loading);
      } else {
        let source_with_loading_matcher = compilation.runtime_template.render(
          &self.template_id(TemplateId::WithLoadingMatcher),
          Some(serde_json::json!({
            "_js_matcher": &has_js_matcher.render("chunkId"),
            "_output_dir": &root_output_dir,
          })),
        )?;

        source.push_str(&source_with_loading_matcher);
      }
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
        "require",
        &compilation.runtime_template,
      )?;
      source.push_str(&hmr_runtime);
    }

    if with_hmr_manifest {
      let source_with_hmr_manifest = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmrManifest), None)?;

      source.push_str(&source_with_hmr_manifest);
    }

    Ok(source)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

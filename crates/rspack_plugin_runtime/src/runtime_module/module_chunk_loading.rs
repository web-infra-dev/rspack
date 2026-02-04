use std::{ptr::NonNull, sync::LazyLock};

use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  BooleanMatcher, Chunk, ChunkGroupOrderKey, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, RuntimeTemplate, RuntimeVariable, compile_boolean_matcher,
  impl_runtime_module,
};
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;

use super::utils::get_output_dir;
use crate::{
  LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper, RuntimePlugin,
  extract_runtime_globals_from_ejs, get_chunk_runtime_requirements,
  runtime_module::{
    generate_javascript_hmr_runtime,
    utils::{get_initial_chunk_ids, stringify_chunks},
  },
};

static MODULE_CHUNK_LOADING_TEMPLATE: &str = include_str!("runtime/module_chunk_loading.ejs");
static MODULE_CHUNK_LOADING_WITH_LOADING_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_loading.ejs");
static MODULE_CHUNK_LOADING_WITH_PREFETCH_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_prefetch.ejs");
static MODULE_CHUNK_LOADING_WITH_PREFETCH_LINK_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_prefetch_link.ejs");
static MODULE_CHUNK_LOADING_WITH_PRELOAD_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_preload.ejs");
static MODULE_CHUNK_LOADING_WITH_PRELOAD_LINK_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_preload_link.ejs");
static MODULE_CHUNK_LOADING_WITH_HMR_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_hmr.ejs");
static MODULE_CHUNK_LOADING_WITH_HMR_MANIFEST_TEMPLATE: &str =
  include_str!("runtime/module_chunk_loading_with_hmr_manifest.ejs");
static JAVASCRIPT_HOT_MODULE_REPLACEMENT_TEMPLATE: &str =
  include_str!("runtime/javascript_hot_module_replacement.ejs");

static MODULE_CHUNK_LOADING_BASIC_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    let mut res = extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_TEMPLATE);
    res.remove(RuntimeGlobals::ON_CHUNKS_LOADED);
    res
  });
static MODULE_CHUNK_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_LOADING_TEMPLATE));
static MODULE_CHUNK_LOADING_WITH_PREFETCH_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_PREFETCH_TEMPLATE)
      | extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_PREFETCH_LINK_TEMPLATE)
  });
static MODULE_CHUNK_LOADING_WITH_PRELOAD_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_PRELOAD_TEMPLATE)
      | extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_PRELOAD_LINK_TEMPLATE)
  });
static MODULE_CHUNK_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_HMR_TEMPLATE));
static MODULE_CHUNK_LOADING_WITH_HMR_MANIFEST_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| {
    extract_runtime_globals_from_ejs(MODULE_CHUNK_LOADING_WITH_HMR_MANIFEST_TEMPLATE)
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
pub struct ModuleChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl ModuleChunkLoadingRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}module_chunk_loading",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }

  pub fn get_runtime_requirements_basic() -> RuntimeGlobals {
    *MODULE_CHUNK_LOADING_BASIC_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_loading() -> RuntimeGlobals {
    *MODULE_CHUNK_LOADING_WITH_LOADING_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_prefetch() -> RuntimeGlobals {
    *MODULE_CHUNK_LOADING_WITH_PREFETCH_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_preload() -> RuntimeGlobals {
    *MODULE_CHUNK_LOADING_WITH_PRELOAD_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_hmr() -> RuntimeGlobals {
    *MODULE_CHUNK_LOADING_WITH_HMR_RUNTIME_REQUIREMENTS
      | *JAVASCRIPT_HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS
  }
  pub fn get_runtime_requirements_with_hmr_manifest() -> RuntimeGlobals {
    *MODULE_CHUNK_LOADING_WITH_HMR_MANIFEST_RUNTIME_REQUIREMENTS
  }
}

impl ModuleChunkLoadingRuntimeModule {
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
          "new URL({}, {}.url);",
          serde_json::to_string(root_output_dir).expect("should able to be serde_json::to_string"),
          compilation.options.output.import_meta_name
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

  fn template(&self, template_id: TemplateId) -> String {
    match template_id {
      TemplateId::Raw => self.id.to_string(),
      TemplateId::WithLoading => format!("{}_with_loading", self.id),
      TemplateId::WithPrefetch => format!("{}_with_prefetch", self.id),
      TemplateId::WithPrefetchLink => format!("{}_with_prefetch_link", self.id),
      TemplateId::WithPreload => format!("{}_with_preload", self.id),
      TemplateId::WithPreloadLink => format!("{}_with_preload_link", self.id),
      TemplateId::WithHMR => format!("{}_with_hmr", self.id),
      TemplateId::WithHMRManifest => format!("{}_with_hmr_manifest", self.id),
      TemplateId::HmrRuntime => format!("{}_hmr_runtime", self.id),
    }
  }
}

enum TemplateId {
  Raw,
  WithLoading,
  WithPrefetch,
  WithPrefetchLink,
  WithPreload,
  WithPreloadLink,
  WithHMR,
  WithHMRManifest,
  HmrRuntime,
}

#[async_trait::async_trait]
impl RuntimeModule for ModuleChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template(TemplateId::Raw),
        MODULE_CHUNK_LOADING_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithLoading),
        MODULE_CHUNK_LOADING_WITH_LOADING_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithPrefetch),
        MODULE_CHUNK_LOADING_WITH_PREFETCH_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithPrefetchLink),
        MODULE_CHUNK_LOADING_WITH_PREFETCH_LINK_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithPreload),
        MODULE_CHUNK_LOADING_WITH_PRELOAD_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithPreloadLink),
        MODULE_CHUNK_LOADING_WITH_PRELOAD_LINK_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithHMR),
        MODULE_CHUNK_LOADING_WITH_HMR_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::WithHMRManifest),
        MODULE_CHUNK_LOADING_WITH_HMR_MANIFEST_TEMPLATE.to_string(),
      ),
      (
        self.template(TemplateId::HmrRuntime),
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

    let hooks = RuntimePlugin::get_compilation_hooks(compilation.id());

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_external_install_chunk =
      runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);

    let is_neutral_platform = compilation.platform.is_neutral();

    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS)
      && (compilation.options.output.environment.supports_document() || is_neutral_platform)
      && chunk.has_child_by_order(
        compilation,
        &ChunkGroupOrderKey::Prefetch,
        true,
        &chunk_has_js,
      );
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS)
      && (compilation.options.output.environment.supports_document() || is_neutral_platform)
      && chunk.has_child_by_order(
        compilation,
        &ChunkGroupOrderKey::Preload,
        true,
        &chunk_has_js,
      );

    let condition_map = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let root_output_dir = get_output_dir(chunk, compilation, true).await?;
    let import_function_name = &compilation.options.output.import_function_name;

    let mut source = String::default();

    if with_base_uri {
      source.push_str(&self.generate_base_uri(chunk, compilation, &root_output_dir));
    } else {
      source.push_str("// no BaseURI")
    }

    source.push_str(&format!(
      r#"
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, Promise] = chunk loading, 0 = chunk loaded
      var installedChunks = {}{};
      "#,
      match with_hmr {
        true => {
          let state_expression = format!(
            "{}_module",
            compilation
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX)
          );
          format!("{state_expression} = {state_expression} || ")
        }
        false => String::new(),
      },
      &stringify_chunks(&initial_chunks, 0)
    ));

    if with_loading || with_external_install_chunk {
      let raw_source = compilation.runtime_template.render(
        &self.template(TemplateId::Raw),
        Some(serde_json::json!({
          "_modules": compilation.runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
          "_with_on_chunk_load": with_on_chunk_load,
        })),
      )?;

      source.push_str(&raw_source);
    } else {
      source.push_str("// no install chunk");
    }

    if with_loading {
      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 0;".to_string()
      } else {
        compilation.runtime_template.render(
          &self.template(TemplateId::WithLoading),
          Some(serde_json::json!({
            "_js_matcher": &has_js_matcher.render("chunkId"),
            "_import_function_name":&compilation.options.output.import_function_name,
            "_output_dir": &root_output_dir,
            "_match_fallback":    if matches!(has_js_matcher, BooleanMatcher::Condition(true)) {
              ""
            } else {
              "else installedChunks[chunkId] = 0;\n"
            },
          })),
        )?
      };

      source.push_str(&format!(
        r#"
        {}.j = function (chunkId, promises) {{
          {body}
        }}
        "#,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      ));
    } else {
      source.push_str("// no chunk on demand loading\n");
    }

    if !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let js_matcher = has_js_matcher.render("chunkId");
      let cross_origin_loading = &compilation.options.output.cross_origin_loading;
      if with_prefetch {
        let link_prefetch_code = compilation.runtime_template.render(
          &self.template(TemplateId::WithPrefetchLink),
          Some(serde_json::json!({
            "_cross_origin": cross_origin_loading.to_string(),
          })),
        )?;

        let chunk_ukey = self.chunk.expect("The chunk should be attached");
        let res = hooks
          .borrow()
          .link_prefetch
          .call(LinkPrefetchData {
            code: link_prefetch_code,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await?;

        let raw_source = compilation.runtime_template.render(
          &self.template(TemplateId::WithPrefetch),
          Some(serde_json::json!({
            "_link_prefetch": &res.code,
            "_js_matcher": &js_matcher,
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;

        source.push_str(&raw_source);
      }
      if with_preload {
        let link_preload_code = compilation.runtime_template.render(
          &self.template(TemplateId::WithPreloadLink),
          Some(serde_json::json!({
            "_cross_origin": cross_origin_loading.to_string(),
          })),
        )?;

        let chunk_ukey = self.chunk.expect("The chunk should be attached");
        let res = hooks
          .borrow()
          .link_preload
          .call(LinkPreloadData {
            code: link_preload_code,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await?;

        let raw_source = compilation.runtime_template.render(
          &self.template(TemplateId::WithPreload),
          Some(serde_json::json!({
            "_js_matcher": &js_matcher,
            "_link_preload": &res.code,
            "_is_neutral_platform": is_neutral_platform
          })),
        )?;

        source.push_str(&raw_source);
      }
    }

    if with_external_install_chunk {
      source.push_str(&format!(
        r#"
        {} = installChunk;
        "#,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::EXTERNAL_INSTALL_CHUNK)
      ));
    } else {
      source.push_str("// no external install chunk\n");
    }

    if with_on_chunk_load {
      source.push_str(&format!(
        r#"
        {}.j = function(chunkId) {{
            return installedChunks[chunkId] === 0;
        }}
        "#,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::ON_CHUNKS_LOADED)
      ));
    } else {
      source.push_str("// no on chunks loaded\n");
    }

    if with_hmr {
      source.push_str(&format!(
        r#"
 {}
 {}
      "#,
        generate_javascript_hmr_runtime(
          &self.template(TemplateId::HmrRuntime),
          "module",
          &compilation.runtime_template
        )?,
        compilation.runtime_template.render(
          &self.template(TemplateId::WithHMR),
          Some(serde_json::json!({
            "_modules": compilation.runtime_template.render_runtime_variable(&RuntimeVariable::Modules),
            "_import_function_name": import_function_name,
          })),
        )?
      ))
    } else {
      source.push_str("// no HMR\n");
    }

    if with_hmr_manifest {
      source.push_str(&compilation.runtime_template.render(
        &self.template(TemplateId::WithHMRManifest),
        Some(serde_json::json!({
          "_import_function_name": import_function_name,
        })),
      )?)
    } else {
      source.push_str("// no HMR manifest\n");
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

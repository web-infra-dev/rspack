use std::ptr::NonNull;

use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  BooleanMatcher, Chunk, ChunkGroupOrderKey, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, RuntimeTemplate, compile_boolean_matcher, impl_runtime_module,
};
use rspack_plugin_javascript::impl_plugin_for_js_plugin::chunk_has_js;

use super::generate_javascript_hmr_runtime;
use crate::{
  LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper, RuntimePlugin,
  get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct JsonpChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl JsonpChunkLoadingRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}jsonp_chunk_loading",
        runtime_template.runtime_module_prefix()
      )),
      None,
    )
  }
}

impl JsonpChunkLoadingRuntimeModule {
  fn generate_base_uri(&self, chunk: &Chunk, compilation: &Compilation) -> String {
    let base_uri = chunk
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| "document.baseURI || self.location.href".to_string());
    format!(
      "{} = {};\n",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::BASE_URI),
      base_uri
    )
  }

  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.as_str();

    match id {
      TemplateId::Raw => base_id.to_string(),
      TemplateId::WithPrefetch => format!("{base_id}_with_prefetch"),
      TemplateId::WithPrefetchLink => format!("{base_id}_with_prefetch_link"),
      TemplateId::WithPreload => format!("{base_id}_with_preload"),
      TemplateId::WithPreloadLink => format!("{base_id}_with_preload_link"),
      TemplateId::WithHmr => format!("{base_id}_with_hmr"),
      TemplateId::WithHmrManifest => format!("{base_id}_with_hmr_manifest"),
      TemplateId::WithOnChunkLoad => format!("{base_id}_with_on_chunk_load"),
      TemplateId::WithCallback => format!("{base_id}_with_callback"),
      TemplateId::HmrRuntime => format!("{base_id}_hmr_runtime"),
    }
  }
}

#[allow(clippy::enum_variant_names)]
enum TemplateId {
  Raw,
  WithPrefetch,
  WithPrefetchLink,
  WithPreload,
  WithPreloadLink,
  WithHmr,
  WithHmrManifest,
  WithOnChunkLoad,
  WithCallback,
  HmrRuntime,
}

#[async_trait::async_trait]
impl RuntimeModule for JsonpChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        include_str!("runtime/jsonp_chunk_loading.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPrefetch),
        include_str!("runtime/jsonp_chunk_loading_with_prefetch.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPrefetchLink),
        include_str!("runtime/jsonp_chunk_loading_with_prefetch_link.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreload),
        include_str!("runtime/jsonp_chunk_loading_with_preload.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithPreloadLink),
        include_str!("runtime/jsonp_chunk_loading_with_preload_link.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmr),
        include_str!("runtime/jsonp_chunk_loading_with_hmr.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithHmrManifest),
        include_str!("runtime/jsonp_chunk_loading_with_hmr_manifest.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithOnChunkLoad),
        include_str!("runtime/jsonp_chunk_loading_with_on_chunk_load.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithCallback),
        include_str!("runtime/jsonp_chunk_loading_with_callback.ejs").to_string(),
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
      .expect_get(&self.chunk.expect("The chunk should be attached"));

    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey());
    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    let with_callback = runtime_requirements.contains(RuntimeGlobals::CHUNK_CALLBACK);
    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS)
      && compilation.options.output.environment.supports_document()
      && chunk.has_child_by_order(
        compilation,
        &ChunkGroupOrderKey::Prefetch,
        true,
        &chunk_has_js,
      );
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS)
      && compilation.options.output.environment.supports_document()
      && chunk.has_child_by_order(
        compilation,
        &ChunkGroupOrderKey::Preload,
        true,
        &chunk_has_js,
      );
    let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);
    let cross_origin_loading = &compilation.options.output.cross_origin_loading;
    let script_type = &compilation.options.output.script_type;

    let hooks = RuntimePlugin::get_compilation_hooks(compilation.id());

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let js_matcher = has_js_matcher.render("chunkId");

    let mut source = String::default();

    if with_base_uri {
      source.push_str(&self.generate_base_uri(chunk, compilation));
    }

    source.push_str(&format!(
      r#"
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
      var installedChunks = {}{};
      "#,
      match with_hmr {
        true => {
          let state_expression = format!(
            "{}_jsonp",
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

    if with_loading {
      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 0;".to_string()
      } else {
        compilation.runtime_template.render(
          &self.template_id(TemplateId::Raw),
          Some(serde_json::json!({
            "_js_matcher": &js_matcher,
            "_fetch_priority": if with_fetch_priority {
               ", fetchPriority"
            } else {
               ""
            },
          })),
        )?
      };

      source.push_str(&format!(
        r#"
        {}.j = function (chunkId, promises{}) {{
          {body}
        }}
        "#,
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::ENSURE_CHUNK_HANDLERS),
        if with_fetch_priority {
          ", fetchPriority"
        } else {
          ""
        },
      ));
    }

    if with_prefetch && !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let link_prefetch_code = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithPrefetchLink),
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

      let source_with_prefetch = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithPrefetch),
        Some(serde_json::json!({
          "_js_matcher": &js_matcher,
          "_link_prefetch": &res.code,
        })),
      )?;

      source.push_str(&source_with_prefetch);
    }

    if with_preload && !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let link_preload_code = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithPreloadLink),
        Some(serde_json::json!({
          "_script_type": script_type.as_str(),
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

      let source_with_preload = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithPreload),
        Some(serde_json::json!({
          "_js_matcher": &js_matcher,
          "_link_preload": &res.code,
        })),
      )?;

      source.push_str(&source_with_preload);
    }

    if with_hmr {
      let source_with_hmr = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmr), Some(serde_json::json!({
          "_global_object": &compilation.options.output.global_object,
          "_hot_update_global": &serde_json::to_string(&compilation.options.output.hot_update_global).expect("failed to serde_json::to_string(hot_update_global)"),
        })))?;

      source.push_str(&source_with_hmr);
      let hmr_runtime = generate_javascript_hmr_runtime(
        &self.template_id(TemplateId::HmrRuntime),
        "jsonp",
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

    if with_on_chunk_load {
      let source_with_on_chunk_load = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithOnChunkLoad), None)?;

      source.push_str(&source_with_on_chunk_load);
    }

    if with_callback || with_loading {
      let chunk_loading_global_expr = format!(
        r#"{}["{}"]"#,
        &compilation.options.output.global_object, &compilation.options.output.chunk_loading_global
      );
      let source_with_callback = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithCallback),
        Some(serde_json::json!({
          "_chunk_loading_global_expr": &chunk_loading_global_expr,
          "_with_on_chunk_load": with_on_chunk_load,
        })),
      )?;

      source.push_str(&source_with_callback);
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

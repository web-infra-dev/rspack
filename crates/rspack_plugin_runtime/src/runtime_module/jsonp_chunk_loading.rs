use std::ptr::NonNull;

use cow_utils::CowUtils;
use pollster::block_on;
use rspack_collections::{DatabaseItem, Identifier};
use rspack_core::{
  compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  BooleanMatcher, Chunk, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage,
};

use super::generate_javascript_hmr_runtime;
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{chunk_has_js, get_initial_chunk_ids, stringify_chunks},
  LinkPrefetchData, LinkPreloadData, RuntimeModuleChunkWrapper, RuntimePlugin,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct JsonpChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for JsonpChunkLoadingRuntimeModule {
  fn default() -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/jsonp_chunk_loading"),
      None,
    )
  }
}

impl JsonpChunkLoadingRuntimeModule {
  fn generate_base_uri(&self, chunk: &Chunk, compilation: &Compilation) -> BoxSource {
    let base_uri = chunk
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| "document.baseURI || self.location.href".to_string());
    RawStringSource::from(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)).boxed()
  }

  fn template_id(&self, id: TemplateId) -> String {
    let base_id = self.id.as_str();

    match id {
      TemplateId::Raw => base_id.to_string(),
      TemplateId::WithPrefetch => format!("{}_with_prefetch", base_id),
      TemplateId::WithPreload => format!("{}_with_preload", base_id),
      TemplateId::WithHmr => format!("{}_with_hmr", base_id),
      TemplateId::WithHmrManifest => format!("{}_with_hmr_manifest", base_id),
      TemplateId::WithOnChunkLoad => format!("{}_with_on_chunk_load", base_id),
      TemplateId::WithCallback => format!("{}_with_callback", base_id),
    }
  }
}

#[allow(clippy::enum_variant_names)]
enum TemplateId {
  Raw,
  WithPrefetch,
  WithPreload,
  WithHmr,
  WithHmrManifest,
  WithOnChunkLoad,
  WithCallback,
}

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
        self.template_id(TemplateId::WithPreload),
        include_str!("runtime/jsonp_chunk_loading_with_preload.ejs").to_string(),
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
    ]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
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
    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS);
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS);
    let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);
    let cross_origin_loading = &compilation.options.output.cross_origin_loading;
    let script_type = &compilation.options.output.script_type;
    let charset = compilation.options.output.charset;

    let hooks = RuntimePlugin::get_compilation_hooks(compilation.id());

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey(), compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let js_matcher = has_js_matcher.render("chunkId");

    let mut source = ConcatSource::default();

    if with_base_uri {
      source.add(self.generate_base_uri(chunk, compilation));
    }

    source.add(RawStringSource::from(format!(
      r#"
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
      var installedChunks = {}{};
      "#,
      match with_hmr {
        true => {
          let state_expression = format!("{}_jsonp", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
          format!("{} = {} || ", state_expression, state_expression)
        }
        false => "".to_string(),
      },
      &stringify_chunks(&initial_chunks, 0)
    )));

    if with_loading {
      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 0;".to_string()
      } else {
        compilation.runtime_template.render(
          &self.template_id(TemplateId::Raw),
          Some(serde_json::json!({
            "_js_matcher": &js_matcher,
            "_match_fallback": if matches!(has_js_matcher, BooleanMatcher::Condition(true)) {
              ""
            } else {
              "else installedChunks[chunkId] = 0;\n"
            },
            "_fetch_priority": if with_fetch_priority {
               ", fetchPriority"
            } else {
               ""
            },
          })),
        )?
      };

      source.add(RawStringSource::from(format!(
        r#"
        {}.j = function (chunkId, promises{}) {{
          {body}
        }}
        "#,
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS,
        if with_fetch_priority {
          ", fetchPriority"
        } else {
          ""
        },
      )));
    }

    if with_prefetch && !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let cross_origin = match cross_origin_loading {
        CrossOriginLoading::Disable => "".to_string(),
        CrossOriginLoading::Enable(_) => {
          format!("link.crossOrigin = {}", cross_origin_loading)
        }
      };
      let link_prefetch_code = r#"
    var link = document.createElement('link');
    $LINK_CHART_CHARSET$
    $CROSS_ORIGIN$
    if (__webpack_require__.nc) {
      link.setAttribute("nonce", __webpack_require__.nc);
    }
    link.rel = "prefetch";
    link.as = "script";
    link.href = __webpack_require__.p + __webpack_require__.u(chunkId);  
      "#
      .cow_replace(
        "$LINK_CHART_CHARSET$",
        if charset {
          "link.charset = 'utf-8';"
        } else {
          ""
        },
      )
      .cow_replace("$CROSS_ORIGIN$", cross_origin.as_str())
      .to_string();

      let chunk_ukey = self.chunk.expect("The chunk should be attached");
      let res = block_on(tokio::task::unconstrained(async {
        hooks
          .link_prefetch
          .call(LinkPrefetchData {
            code: link_prefetch_code,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await
      }))?;

      let source_with_prefetch = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithPrefetch),
        Some(serde_json::json!({
          "_js_matcher": &js_matcher,
          "_link_prefetch": &res.code,
        })),
      )?;

      source.add(RawStringSource::from(source_with_prefetch));
    }

    if with_preload && !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let cross_origin = match cross_origin_loading {
        CrossOriginLoading::Disable => "".to_string(),
        CrossOriginLoading::Enable(cross_origin_value) => {
          if cross_origin_value.eq("use-credentials") {
            "link.crossOrigin = \"use-credentials\";".to_string()
          } else {
            format!(
              r#"
              if (link.href.indexOf(window.location.origin + '/') !== 0) {{
                link.crossOrigin = {}
              }}
              "#,
              cross_origin_loading
            )
          }
        }
      };
      let script_type_link_pre = if script_type.eq("module") || script_type.eq("false") {
        "".to_string()
      } else {
        format!(
          "link.type = {}",
          serde_json::to_string(script_type).expect("invalid json tostring")
        )
      };
      let script_type_link_post = if script_type.eq("module") {
        "link.rel = \"modulepreload\";"
      } else {
        r#"
        link.rel = "preload";
        link.as = "script";
        "#
      };

      let link_preload_code = r#"
    var link = document.createElement('link');
    $LINK_CHART_CHARSET$
    $SCRIPT_TYPE_LINK_PRE$
    if (__webpack_require__.nc) {
      link.setAttribute("nonce", __webpack_require__.nc);
    }
    $SCRIPT_TYPE_LINK_POST$
    link.href = __webpack_require__.p + __webpack_require__.u(chunkId);
    $CROSS_ORIGIN$  
      "#
      .cow_replace(
        "$LINK_CHART_CHARSET$",
        if charset {
          "link.charset = 'utf-8';"
        } else {
          ""
        },
      )
      .cow_replace("$CROSS_ORIGIN$", cross_origin.as_str())
      .cow_replace("$SCRIPT_TYPE_LINK_PRE$", script_type_link_pre.as_str())
      .cow_replace("$SCRIPT_TYPE_LINK_POST$", script_type_link_post)
      .to_string();

      let chunk_ukey = self.chunk.expect("The chunk should be attached");
      let res = block_on(tokio::task::unconstrained(async {
        hooks
          .link_preload
          .call(LinkPreloadData {
            code: link_preload_code,
            chunk: RuntimeModuleChunkWrapper {
              chunk_ukey,
              compilation_id: compilation.id(),
              compilation: NonNull::from(compilation),
            },
          })
          .await
      }))?;

      let source_with_preload = compilation.runtime_template.render(
        &self.template_id(TemplateId::WithPreload),
        Some(serde_json::json!({
          "_js_matcher": &js_matcher,
          "_link_preload": &res.code,
        })),
      )?;

      source.add(RawStringSource::from(source_with_preload));
    }

    if with_hmr {
      let source_with_hmr = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmr), Some(serde_json::json!({
          "_global_object": &compilation.options.output.global_object,
          "_hot_update_global": &serde_json::to_string(&compilation.options.output.hot_update_global).expect("failed to serde_json::to_string(hot_update_global)"),
        })))?;

      source.add(RawStringSource::from(source_with_hmr));
      source.add(RawStringSource::from(generate_javascript_hmr_runtime(
        "jsonp",
      )));
    }

    if with_hmr_manifest {
      let source_with_hmr_manifest = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithHmrManifest), None)?;

      source.add(RawStringSource::from(source_with_hmr_manifest));
    }

    if with_on_chunk_load {
      let source_with_on_chunk_load = compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithOnChunkLoad), None)?;

      source.add(RawStringSource::from(source_with_on_chunk_load));
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
          "_with_on_chunk_load": match with_on_chunk_load {
            true => format!("return {}(result);", RuntimeGlobals::ON_CHUNKS_LOADED.name()),
            false => "".to_string(),
          },
        })),
      )?;

      source.add(RawStringSource::from(source_with_callback));
    }

    Ok(source.boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

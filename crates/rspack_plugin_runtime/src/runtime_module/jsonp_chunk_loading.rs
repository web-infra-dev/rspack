use rspack_core::{
  compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  BooleanMatcher, Chunk, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage,
};
use rspack_identifier::Identifier;

use super::generate_javascript_hmr_runtime;
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{chunk_has_js, get_initial_chunk_ids, stringify_chunks},
};

#[impl_runtime_module]
#[derive(Debug, Eq)]
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
    RawSource::from(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)).boxed()
  }
}

impl RuntimeModule for JsonpChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached"));

    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey);
    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    let with_callback = runtime_requirements.contains(RuntimeGlobals::CHUNK_CALLBACK);
    let with_prefetch = runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS);
    let with_preload = runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS);
    let cross_origin_loading = &compilation.options.output.cross_origin_loading;
    let script_type = &compilation.options.output.script_type;

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey, compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let js_matcher = has_js_matcher.render("chunkId");

    let mut source = ConcatSource::default();

    if with_base_uri {
      source.add(self.generate_base_uri(chunk, compilation));
    }

    source.add(RawSource::from(format!(
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
        include_str!("runtime/jsonp_chunk_loading.js")
          .replace("$JS_MATCHER$", &js_matcher)
          .replace(
            "$MATCH_FALLBACK$",
            if matches!(has_js_matcher, BooleanMatcher::Condition(true)) {
              ""
            } else {
              "else installedChunks[chunkId] = 0;\n"
            },
          )
      };

      source.add(RawSource::from(format!(
        r#"
        {}.j = function (chunkId, promises) {{
          {body}
        }}
        "#,
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS
      )));
    }

    if with_prefetch && !matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
      let cross_origin = match cross_origin_loading {
        CrossOriginLoading::Disable => "".to_string(),
        CrossOriginLoading::Enable(_) => {
          format!("link.crossOrigin = {}", cross_origin_loading)
        }
      };
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_prefetch.js")
          .replace("$JS_MATCHER$", &js_matcher)
          .replace("$CROSS_ORIGIN$", cross_origin.as_str()),
      ));
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

      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_preload.js")
          .replace("$JS_MATCHER$", &js_matcher)
          .replace("$CROSS_ORIGIN$", cross_origin.as_str())
          .replace("$SCRIPT_TYPE_LINK_PRE$", script_type_link_pre.as_str())
          .replace("$SCRIPT_TYPE_LINK_POST$", script_type_link_post),
      ));
    }

    if with_hmr {
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_hmr.js")
          .replace("$GLOBAL_OBJECT$", &compilation.options.output.global_object)
          .replace(
            "$HOT_UPDATE_GLOBAL$",
            &serde_json::to_string(&compilation.options.output.hot_update_global)
              .expect("failed to serde_json::to_string(hot_update_global)"),
          ),
      ));
      source.add(RawSource::from(generate_javascript_hmr_runtime("jsonp")));
    }

    if with_hmr_manifest {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_hmr_manifest.js"
      )));
    }

    if with_on_chunk_load {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if with_callback || with_loading {
      let chunk_loading_global_expr = format!(
        r#"{}["{}"]"#,
        &compilation.options.output.global_object, &compilation.options.output.chunk_loading_global
      );
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_callback.js")
          .replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr)
          .replace(
            "$WITH_ON_CHUNK_LOAD$",
            match with_on_chunk_load {
              true => "return __webpack_require__.O(result);",
              false => "",
            },
          ),
      ));
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

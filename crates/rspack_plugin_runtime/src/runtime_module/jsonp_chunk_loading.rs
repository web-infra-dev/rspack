use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;

use super::BooleanMatcher;
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{
    chunk_has_js, get_initial_chunk_ids, render_condition_map, stringify_chunks,
  },
};

#[derive(Debug, Eq)]
pub struct JsonpChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for JsonpChunkLoadingRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/jsonp_chunk_loading"),
      chunk: None,
    }
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

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk = compilation
      .chunk_by_ukey
      .get(&self.chunk.expect("The chunk should be attached"))
      .expect("should have chunk");

    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey);
    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    let with_callback = runtime_requirements.contains(RuntimeGlobals::CHUNK_CALLBACK);

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey, compilation, chunk_has_js);
    let has_js_matcher = render_condition_map(&condition_map, "chunkId");
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

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
          .replace("$JS_MATCHER$", has_js_matcher.to_string().as_str())
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
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
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

    source.boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

impl_runtime_module!(JsonpChunkLoadingRuntimeModule);

use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;

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
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let chunk = compilation
      .chunk_by_ukey
      .get(&self.chunk.expect("The chunk should be attached"))
      .expect("should have chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey);
    let mut source = ConcatSource::default();

    if runtime_requirements.contains(RuntimeGlobals::BASE_URI) {
      source.add(self.generate_base_uri(chunk, compilation));
    }

    // object to store loaded and loading chunks
    // undefined = chunk not loaded, null = chunk preloaded/prefetched
    // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
    source.add(RawSource::from(format!(
      "var installedChunks = {};\n",
      &stringify_chunks(&initial_chunks, 0)
    )));

    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);

    if with_loading {
      let condition_map =
        compilation
          .chunk_graph
          .get_chunk_condition_map(&chunk.ukey, compilation, chunk_has_js);
      // If chunkId not corresponding chunkName will skip load it.
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading.js").replace(
          "JS_MATCHER",
          &render_condition_map(&condition_map, "chunkId").to_string(),
        ),
      ));
    }

    if runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS) {
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_hmr.js")
          .replace("$globalObject$", &compilation.options.output.global_object)
          .replace(
            "$hotUpdateGlobal$",
            &serde_json::to_string(&compilation.options.output.hot_update_global)
              .expect("failed to serde_json::to_string(hot_update_global)"),
          ),
      ));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
    }

    if runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST) {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_hmr_manifest.js"
      )));
    }

    if with_on_chunk_load {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if runtime_requirements.contains(RuntimeGlobals::CHUNK_CALLBACK) || with_loading {
      let chunk_loading_global_expr = format!(
        "{}['{}']",
        &compilation.options.output.global_object, &compilation.options.output.chunk_loading_global
      );
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_callback.js")
          .replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr)
          .replace(
            "$withOnChunkLoad$",
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

use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH,
};
use rspack_identifier::Identifier;

use super::utils::chunk_has_js;
use crate::impl_runtime_module;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default, Eq)]
pub struct JsonpChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  runtime_requirements: RuntimeGlobals,
}

impl JsonpChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/jsonp_chunk_loading"),
      chunk: None,
      runtime_requirements,
    }
  }
}

impl RuntimeModule for JsonpChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    // let condition_map = compilation.chunk_graph.get_chunk_condition_map(
    //   &chunk_ukey,
    //   &compilation.chunk_by_ukey,
    //   &compilation.chunk_group_by_ukey,
    //   &compilation.module_graph,
    //   chunk_hash_js,
    // );
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let mut source = ConcatSource::default();

    if self.runtime_requirements.contains(RuntimeGlobals::BASE_URI) {
      source.add(RawSource::from(format!(
        "{} = document.baseURI || self.location.href;\n",
        RuntimeGlobals::BASE_URI
      )))
    }

    // object to store loaded and loading chunks
    // undefined = chunk not loaded, null = chunk preloaded/prefetched
    // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
    source.add(RawSource::from(format!(
      "var installedChunks = {};\n",
      &stringify_chunks(&initial_chunks, 0)
    )));

    let with_loading = self
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = self
      .runtime_requirements
      .contains(RuntimeGlobals::ON_CHUNKS_LOADED);

    if with_loading {
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId"),
      ));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
    {
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading_with_hmr.js")
          .replace("$globalObject$", &compilation.options.output.global_object),
      ));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST)
    {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_hmr_manifest.js"
      )));
    }

    if with_on_chunk_load {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::CHUNK_CALLBACK)
      || with_loading
    {
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

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(JsonpChunkLoadingRuntimeModule);

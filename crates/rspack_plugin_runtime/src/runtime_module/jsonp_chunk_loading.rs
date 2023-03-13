use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  runtime_globals, ChunkUkey, Compilation, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH,
};
use rustc_hash::FxHashSet as HashSet;

use super::utils::chunk_has_js;
use crate::impl_runtime_module;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default, Eq)]
pub struct JsonpChunkLoadingRuntimeModule {
  chunk: Option<ChunkUkey>,
  runtime_requirements: HashSet<&'static str>,
}

impl JsonpChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: HashSet<&'static str>) -> Self {
    Self {
      chunk: None,
      runtime_requirements,
    }
  }
}

impl RuntimeModule for JsonpChunkLoadingRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/jsonp_chunk_loading".to_owned()
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
    // object to store loaded and loading chunks
    // undefined = chunk not loaded, null = chunk preloaded/prefetched
    // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
    source.add(RawSource::from(format!(
      "var installedChunks = {};\n",
      &stringify_chunks(&initial_chunks, 0)
    )));

    let with_loading = self
      .runtime_requirements
      .contains(runtime_globals::ENSURE_CHUNK_HANDLERS);

    if with_loading {
      source.add(RawSource::from(
        include_str!("runtime/jsonp_chunk_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId"),
      ));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS)
    {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_hmr.js"
      )));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::HMR_DOWNLOAD_MANIFEST)
    {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_hmr_manifest.js"
      )));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::ON_CHUNKS_LOADED)
    {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::CHUNK_CALLBACK)
      || with_loading
    {
      source.add(RawSource::from(include_str!(
        "runtime/jsonp_chunk_loading_with_callback.js"
      )));
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

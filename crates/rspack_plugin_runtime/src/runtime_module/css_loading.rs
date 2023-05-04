use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkGraph, ChunkUkey, Compilation, ModuleGraph, RuntimeGlobals, RuntimeModule, SourceType,
  RUNTIME_MODULE_STAGE_ATTACH,
};
use rspack_identifier::Identifier;
use rspack_plugin_javascript::runtime::stringify_chunks_to_array;
use rustc_hash::FxHashSet as HashSet;

use crate::impl_runtime_module;
use crate::runtime_module::stringify_chunks;
#[derive(Debug, Default, Eq)]
pub struct CssLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  runtime_requirements: RuntimeGlobals,
}

impl CssLoadingRuntimeModule {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/css_loading"),
      chunk: None,
      runtime_requirements,
    }
  }

  pub fn chunk_has_css(
    chunk: &ChunkUkey,
    chunk_graph: &ChunkGraph,
    module_graph: &ModuleGraph,
  ) -> bool {
    !chunk_graph
      .get_chunk_modules_by_source_type(chunk, SourceType::Css, module_graph)
      .is_empty()
  }
}

impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation
        .chunk_by_ukey
        .get(&chunk_ukey)
        .expect("Chunk not found");

      let all_async_chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);
      let mut async_chunk_ids_with_css = HashSet::default();
      for chunk_ukey in all_async_chunks.iter() {
        if Self::chunk_has_css(
          chunk_ukey,
          &compilation.chunk_graph,
          &compilation.module_graph,
        ) {
          let chunk = compilation
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("Chunk not found");
          async_chunk_ids_with_css.insert(chunk.expect_id().to_string());
        }
      }

      let with_hmr = self
        .runtime_requirements
        .contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

      let with_loading = self
        .runtime_requirements
        .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);

      if !with_hmr && !with_loading && async_chunk_ids_with_css.is_empty() {
        return RawSource::from("").boxed();
      }

      let mut initial_chunk_ids_with_css = HashSet::default();
      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);

      for chunk_ukey in initial_chunks.iter() {
        if Self::chunk_has_css(
          chunk_ukey,
          &compilation.chunk_graph,
          &compilation.module_graph,
        ) {
          let chunk = compilation
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("Chunk not found");
          initial_chunk_ids_with_css.insert(chunk.expect_id().to_string());
        }
      }

      let mut source = ConcatSource::default();
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunk_ids_with_css, 0)
      )));

      source.add(RawSource::from(
        include_str!("runtime/css_loading.js").replace(
          "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
          &compilation.options.output.cross_origin_loading.to_string(),
        ),
      ));

      if with_loading {
        source.add(RawSource::from(
          include_str!("runtime/css_loading_with_loading.js").replace(
            "CSS_MATCHER",
            format!(
              "{}.indexOf(chunkId) > -1",
              stringify_chunks_to_array(&async_chunk_ids_with_css)
            )
            .as_str(),
          ),
        ));
      }

      if with_hmr {
        source.add(RawSource::from(include_str!(
          "runtime/css_loading_with_hmr.js"
        )));
      }

      source.boxed()
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(CssLoadingRuntimeModule);

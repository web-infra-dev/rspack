use hashbrown::HashSet;
use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  runtime_globals, ChunkUkey, Compilation, RuntimeModule, SourceType,
};

use super::utils::stringify_chunks;

#[derive(Debug, Default)]
pub struct CssLoadingRuntimeModule {
  chunk: Option<ChunkUkey>,
  runtime_requirements: HashSet<String>,
}

impl CssLoadingRuntimeModule {
  pub fn new(runtime_requirements: HashSet<String>) -> Self {
    Self {
      chunk: None,
      runtime_requirements,
    }
  }

  pub fn chunk_has_css(chunk: &ChunkUkey, compilation: &Compilation) -> bool {
    !compilation
      .chunk_graph
      .get_chunk_modules_by_source_type(chunk, SourceType::Css, &compilation.module_graph)
      .is_empty()
  }
}

impl RuntimeModule for CssLoadingRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/css_loading".to_string()
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation
        .chunk_by_ukey
        .get(&chunk_ukey)
        .expect("Chunk not found");

      let all_async_chunks = chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey);

      let with_hmr = self
        .runtime_requirements
        .contains(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS);

      let with_loading = self
        .runtime_requirements
        .contains(runtime_globals::ENSURE_CHUNK_HANDLERS);

      if !with_hmr
        && !with_loading
        && all_async_chunks
          .iter()
          .all(|chunk_ukey| !Self::chunk_has_css(chunk_ukey, compilation))
      {
        return RawSource::from("".to_string()).boxed();
      }

      let mut initial_chunk_ids_with_css = HashSet::new();
      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);

      for chunk_ukey in initial_chunks.iter() {
        if Self::chunk_has_css(chunk_ukey, compilation) {
          let chunk = compilation
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("Chunk not found");
          initial_chunk_ids_with_css.insert(chunk.id.clone());
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

      source.add(RawSource::from(include_str!("runtime/css_loading.js")));

      if with_loading {
        source.add(RawSource::from(
          include_str!("runtime/css_loading_with_loading.js")
            // TODO
            .replace("CSS_MATCHER", "chunkId"),
        ));
      }

      if with_hmr {
        source.add(RawSource::from(
          include_str!("runtime/css_loading_with_hmr.js").to_string(),
        ));
      }
      source.boxed()
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

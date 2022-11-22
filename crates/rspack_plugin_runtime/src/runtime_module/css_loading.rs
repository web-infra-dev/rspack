use hashbrown::HashSet;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule, SourceType,
};

use super::utils::stringify_chunks;

#[derive(Debug, Default)]
pub struct CssLoadingRuntimeModule {
  chunk: Option<ChunkUkey>,
}

impl CssLoadingRuntimeModule {
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

      if all_async_chunks
        .iter()
        .all(|chunk_ukey| !Self::chunk_has_css(chunk_ukey, compilation))
      {
        return RawSource::from("".to_string()).boxed();
      }

      let mut initial_chunk_ids_with_css = HashSet::new();

      for chunk_ukey in chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey) {
        if Self::chunk_has_css(&chunk_ukey, compilation) {
          let chunk = compilation
            .chunk_by_ukey
            .get(&chunk_ukey)
            .expect("Chunk not found");
          initial_chunk_ids_with_css.insert(chunk.id.clone());
        }
      }
      RawSource::from(
        include_str!("runtime/css_loading.js")
          .to_string()
          .replace(
            "INSTALLED_CHUNKS_WITH_CSS",
            &stringify_chunks(&initial_chunk_ids_with_css, 0),
          )
          // TODO
          .replace("CSS_MATCHER", "chunkId"),
      )
      .boxed()
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

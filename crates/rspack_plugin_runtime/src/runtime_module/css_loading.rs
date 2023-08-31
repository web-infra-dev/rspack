use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH,
};
use rspack_identifier::Identifier;
use rustc_hash::FxHashSet as HashSet;

use super::utils::chunk_has_css;
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

      let with_hmr = self
        .runtime_requirements
        .contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

      let with_loading = self
        .runtime_requirements
        .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);

      let mut initial_chunk_ids_with_css = HashSet::default();
      let mut initial_chunk_ids_without_css = HashSet::default();
      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);

      for chunk_ukey in initial_chunks.iter() {
        let chunk = compilation
          .chunk_by_ukey
          .get(chunk_ukey)
          .expect("Chunk not found");
        if chunk_has_css(chunk_ukey, compilation) {
          initial_chunk_ids_with_css.insert(chunk.expect_id().to_string());
        } else {
          initial_chunk_ids_without_css.insert(chunk.expect_id().to_string());
        }
      }

      if !with_hmr && !with_loading && initial_chunk_ids_with_css.is_empty() {
        return RawSource::from("").boxed();
      }

      let mut source = ConcatSource::default();
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded

      // One entry initial chunk maybe is other entry dynamic chunk, so here
      // only render chunk without css. See packages/rspack/tests/runtimeCases/runtime/split-css-chunk test.
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunk_ids_without_css, 0)
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
            &format!("{}(chunkId)", RuntimeGlobals::GET_CHUNK_CSS_FILENAME),
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

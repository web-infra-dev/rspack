use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{impl_runtime_module, ChunkUkey, Compilation, RuntimeModule};
use rspack_identifier::Identifier;
use rspack_plugin_javascript::runtime::stringify_array;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct LoadChunkWithBlockRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for LoadChunkWithBlockRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/load_chunk_with_block"),
      chunk: None,
      source_map_kind: SourceMapKind::None,
      source_map_columns: true,
    }
  }
}

impl RuntimeModule for LoadChunkWithBlockRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk_ukey = self.chunk.expect("should have chunk");
    let runtime = &compilation.chunk_by_ukey.expect_get(&chunk_ukey).runtime;

    let blocks = compilation
      .module_graph
      .modules()
      .par_iter()
      .map(|(_, module)| module.get_blocks())
      .flatten()
      .collect::<HashSet<_>>();
    let map = blocks
      .par_iter()
      .filter_map(|block_id| {
        let chunk_group = compilation
          .chunk_graph
          .get_block_chunk_group(block_id, &compilation.chunk_group_by_ukey)?;
        let chunk_ids = chunk_group
          .chunks
          .iter()
          .filter_map(|chunk_ukey| {
            let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
            if chunk.runtime.is_superset(runtime) {
              Some(chunk.expect_id().to_string())
            } else {
              None
            }
          })
          .collect::<Vec<_>>();
        if chunk_ids.is_empty() {
          return None;
        }
        let block = compilation
          .module_graph
          .block_by_id(block_id)
          .expect("should have block");
        let key = block.block_promise_key(compilation);
        Some((key, chunk_ids))
      })
      .collect::<HashMap<String, Vec<String>>>();

    let mut source = ConcatSource::default();
    source.add(RawSource::from(format!(
      "var map = {};\n",
      &stringify_map(&map)
    )));
    source.add(RawSource::from(
      "
__webpack_require__.el = function(module) {
  var chunkIds = map[module];
  if (chunkIds === undefined) return Promise.resolve();
  if (chunkIds.length > 1) return Promise.all(chunkIds.map(__webpack_require__.e));
  return __webpack_require__.e(chunkIds[0]);
}
",
    ));

    source.boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

fn stringify_map(map: &HashMap<String, Vec<String>>) -> String {
  format!(
    r#"{{{}}}"#,
    map
      .keys()
      .sorted_unstable()
      .map(|key| {
        format!(
          r#"{}: {}"#,
          key,
          stringify_array(map.get(key).expect("get key from map"))
        )
      })
      .join(", ")
  )
}

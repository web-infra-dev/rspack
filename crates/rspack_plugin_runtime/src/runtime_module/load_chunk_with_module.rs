use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_plugin_javascript::runtime::stringify_array;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct LoadChunkWithModuleRuntimeModule {
  id: Identifier,
}

impl Default for LoadChunkWithModuleRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/load_chunk_with_module"),
    }
  }
}

impl RuntimeModule for LoadChunkWithModuleRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let async_modules = compilation
      .module_graph
      .module_graph_modules()
      .par_iter()
      .map(|(_, mgm)| mgm.dynamic_depended_modules(&compilation.module_graph))
      .flatten()
      .map(|(id, _)| id)
      .collect::<HashSet<_>>();
    let map = async_modules
      .par_iter()
      .map(|identifier| {
        let mut chunk_ids = {
          let chunk_group = compilation
            .chunk_graph
            .get_block_chunk_group(identifier, &compilation.chunk_group_by_ukey);
          chunk_group
            .chunks
            .iter()
            .map(|chunk_ukey| {
              let chunk = compilation
                .chunk_by_ukey
                .get(chunk_ukey)
                .expect("chunk should exist");
              chunk.expect_id().to_string()
            })
            .collect::<Vec<_>>()
        };
        chunk_ids.sort_unstable();
        let module = compilation
          .module_graph
          .module_graph_module_by_identifier(identifier)
          .expect("no module found");
        (module.id(&compilation.chunk_graph).to_string(), chunk_ids)
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
        var chunkId = map[module];
        if (chunkId === undefined) {
            return Promise.resolve();
        }
        if (chunkId.length > 1) {
          return Promise.all(chunkId.map(__webpack_require__.e));
        } else {
          return __webpack_require__.e(chunkId[0]);
        };
    }
    ",
    ));

    source.boxed()
  }
}

impl_runtime_module!(LoadChunkWithModuleRuntimeModule);

fn stringify_map(map: &HashMap<String, Vec<String>>) -> String {
  format!(
    r#"{{{}}}"#,
    map.keys().sorted().fold(String::new(), |prev, cur| {
      prev
        + format!(
          r#""{}": {},"#,
          cur,
          stringify_array(map.get(cur).expect("get key from map"))
        )
        .as_str()
    })
  )
}

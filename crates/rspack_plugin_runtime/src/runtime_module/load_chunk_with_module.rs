use itertools::Itertools;
use rayon::prelude::*;
use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_plugin_javascript::runtime::stringify_array;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct LoadChunkWithModuleRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for LoadChunkWithModuleRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/load_chunk_with_module"),
      chunk: None,
    }
  }
}

impl RuntimeModule for LoadChunkWithModuleRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    if let Some(chunk_ukey) = &self.chunk {
      let runtime = &compilation
        .chunk_by_ukey
        .get(chunk_ukey)
        .expect("should have chunk")
        .runtime;

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
              .filter_map(|chunk_ukey| {
                let chunk = compilation
                  .chunk_by_ukey
                  .get(chunk_ukey)
                  .expect("chunk should exist");
                if chunk.runtime.is_superset(runtime) {
                  Some(chunk.expect_id().to_string())
                } else {
                  None
                }
              })
              .collect::<Vec<_>>()
          };
          chunk_ids.sort_unstable();
          let module = compilation
            .module_graph
            .module_graph_module_by_identifier(identifier)
            .expect("no module found");

          let module_id = module.id(&compilation.chunk_graph);
          let module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");

          // TODO if chunk_ids is empty, here should remove the filed
          (module_id_expr, chunk_ids)
        })
        .collect::<HashMap<String, Vec<String>>>();

      let mut source = ConcatSource::default();
      source.add(RawSource::from(format!(
        "var map = {};\n",
        &stringify_map(&map)
      )));
      source.add(RawSource::from(
        "__webpack_require__.el = function(module) {
          var chunkId = map[module];
          if (chunkId === undefined) {
              return Promise.resolve();
          }
          if (chunkId.length > 1) {
            return Promise.all(chunkId.map(__webpack_require__.e));
          } else {
            return __webpack_require__.e(chunkId[0]);
          };
      }\n",
      ));

      source.boxed()
    } else {
      unreachable!("should have chunk")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(LoadChunkWithModuleRuntimeModule);

fn stringify_map(map: &HashMap<String, Vec<String>>) -> String {
  format!(
    r#"{{{}}}"#,
    map.keys().sorted().fold(String::new(), |prev, cur| {
      prev
        + format!(
          r#"{}: {},"#,
          cur,
          stringify_array(map.get(cur).expect("get key from map"))
        )
        .as_str()
    })
  )
}

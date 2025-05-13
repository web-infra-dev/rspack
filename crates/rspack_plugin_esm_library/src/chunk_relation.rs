use rspack_collections::{IdentifierMap, IdentifierSet, UkeyMap};
use rspack_core::{ChunkLink, ChunkUkey, Compilation, ConcatenationScope};
use rspack_error::Result;
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet},
};

use crate::EsmLibraryPlugin;

impl EsmLibraryPlugin {
  pub(crate) async fn calculate_chunk_relation(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = compilation.get_module_graph();
    let all_chunks: Vec<ChunkUkey> = compilation.chunk_by_ukey.keys().copied().collect();
    let concate_modules_map = self.concatenated_modules_map.lock().await;
    let concate_modules_map = concate_modules_map
      .get(&compilation.id().0)
      .expect("should has compilation");
    let mut link = UkeyMap::<ChunkUkey, ChunkLink>::default();
    let mut record_exports = UkeyMap::<ChunkUkey, IdentifierMap<FxHashSet<Atom>>>::default();

    // calculate imports to other chunks
    for chunk_ukey in &all_chunks {
      link.entry(*chunk_ukey).or_default();

      let all_chunk_modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier(&chunk_ukey)
        .iter()
        .filter(|m| concate_modules_map.contains_key(*m))
        .copied()
        .collect::<IdentifierSet>();

      let mut chunk_modules = all_chunk_modules.iter().copied().collect::<Vec<_>>();

      chunk_modules.sort_by(|m1, m2| {
        let m1_index = module_graph.get_post_order_index(m1);
        let m2_index = module_graph.get_post_order_index(m2);
        m1_index.cmp(&m2_index)
      });

      let chunk_link = link.get_mut(&chunk_ukey).expect("should have chunk link");
      let mut errors = vec![];

      for m in chunk_modules {
        let module = module_graph
          .module_by_identifier(&m)
          .expect("should have module");
        let codegen_res = compilation.code_generation_results.get(&m, None);
        let Some(concatenation_scope) = &codegen_res.concatenation_scope else {
          continue;
        };
        let imports = chunk_link.imports.entry(*chunk_ukey).or_default();

        for (imported, refs) in &concatenation_scope.refs {
          if all_chunk_modules.contains(imported) {
            continue;
          }
          let import_refs = imports.entry(*imported).or_default();

          let chunk = compilation.chunk_graph.get_module_chunks(*imported);
          if chunk.len() > 1 {
            errors.push(format!("module exist in multiple chunks {}", imported));
            continue;
          }

          if chunk.is_empty() {
            errors.push(format!("module not exist in any chunk {}", imported));
            continue;
          }

          let chunk_ukey = chunk
            .into_iter()
            .next()
            .expect("should have at least one chunk");
          let exports = record_exports.entry(*chunk_ukey).or_default();
          let exports = exports.entry(*imported).or_default();

          let imported_exports_info = module_graph.get_exports_info(imported);

          for import_ref in refs {
            let match_ref = ConcatenationScope::match_module_reference(&import_ref)
              .expect("should have exact match");

            let imported_name = &match_ref.ids[0];
            import_refs.insert(imported_name.clone());
            exports.insert(imported_name.clone());
          }
        }
      }
    }

    // record exports
    for (chunk_ukey, exports) in record_exports {
      let chunk_link = link.entry(chunk_ukey).or_default();
      chunk_link.exports = exports;
    }

    compilation.chunk_graph.link = Some(link);
    Ok(())
  }
}

use std::collections::{HashMap, HashSet, VecDeque};

// use crate::{
//     BundleOptions, Chunk, ChunkGraph, ChunkIdAlgo, ChunkKind, JsModuleKind, ModuleGraphContainer,
// };

use crate::{
  uri_to_chunk_name, Chunk, ChunkIdAlgo, ChunkKind, ChunkUkey, Compilation, ModuleGraph,
};

pub fn code_splitting2(compilation: &mut Compilation) {
  // let code_splitting_options = &bundle_options.code_splitting;
  let is_enable_code_splitting = true;
  let is_reuse_existing_chunk = true;

  let module_graph = &compilation.module_graph;

  let mut id_generator = ChunkIdGenerator {
    id_count: 0,
    module_graph,
    root: compilation.options.context.as_str(),
    chunk_id_algo: ChunkIdAlgo::Named,
  };

  let mut chunk_ref_by_entry_module_uri: HashMap<&str, ChunkUkey> = HashMap::new();
  let mut chunk_relation_graph2 = petgraph::graphmap::DiGraphMap::<ChunkUkey, ()>::new();

  let mut chunk_entries = compilation
    .entry_dependencies()
    .values()
    .filter_map(|dep| module_graph.module_by_dependency(dep))
    .map(|module| module.uri.as_str())
    .collect::<Vec<_>>();

  let chunk_by_ref = &mut compilation.chunk_by_ukey;

  let chunk_graph = &mut compilation.chunk_graph;

  // First we need to create entry chunk.
  for entry in &chunk_entries {
    let chunk_id = id_generator.gen_id(entry);
    let chunk = Chunk::new(
      ChunkUkey::with_debug_info("Entry chunk"),
      chunk_id.clone(),
      entry.to_string(),
      ChunkKind::Entry { name: chunk_id },
    );

    chunk_ref_by_entry_module_uri.insert(*entry, chunk.rid);
    chunk_graph.add_chunk(&chunk);
    chunk_by_ref.insert(chunk.rid, chunk);
  }

  if is_enable_code_splitting {
    module_graph.modules().for_each(|module| {
      module
        .dynamic_depended_modules(module_graph)
        .into_iter()
        .for_each(|dyn_dep_module| {
          chunk_ref_by_entry_module_uri
            .entry(dyn_dep_module.uri.as_str())
            .or_insert_with_key(|mod_uri| {
              chunk_entries.push(*mod_uri);

              let chunk_id = id_generator.gen_id(mod_uri);
              let chunk = Chunk::new(
                ChunkUkey::with_debug_info("Async chunk"),
                chunk_id,
                mod_uri.to_string(),
                ChunkKind::Normal,
              );
              let chunk_ref = chunk.rid;
              chunk_graph.add_chunk(&chunk);
              chunk_by_ref.insert(chunk.rid, chunk);
              chunk_ref
            });
        });
    });
  }

  // Now, we have all chunks and need place right modules into chunks.
  // We iterate through all chunks, and place modules that depended(directed or non-directed) by the chunk to the map below.
  // Without bundle splitting, a module can be placed into multiple chunks based on its usage:

  // E.g. (Code Splitting enabled)
  //                  (dynamic import)
  // a.js(entrypoint)------------------>b.js------->c.js
  //        |
  //        |
  //        +------>c.js
  // In this case, two chunks will be generated, chunk entires are `a.js` (Chunk A) and `b.js` (Chunk B),
  // and module `c.js` will be placed into both of them.

  let mut mod_to_chunk_ref: HashMap<&str, HashSet<ChunkUkey>> = Default::default();

  for entry in &chunk_entries {
    let chunk_ref = chunk_ref_by_entry_module_uri[*entry];
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_graph
        .module_by_uri(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);
        mod_to_chunk_ref
          .entry(module_uri)
          .or_default()
          .insert(chunk_ref);
        // let chunk = &mut chunk_graph[chunk_id];
        // chunk.module_ids.push(module_uri.to_string());
        module
          .depended_modules(module_graph)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_splitting {
          module
            .dynamic_depended_modules(module_graph)
            .into_iter()
            .for_each(|module| queue.push_back(&module.uri));
        }
      } else {
        // TODO: detect circle import
      }
    }
  }

  // Now, we have the relationship between modules and chunks.
  // We create directed graph from starting chunk to another.

  // For the example above, we have the following graph:
  // Chunk A(entrypoint: a.js) -> Chunk B(entrypoint: b.js)

  module_graph.modules().for_each(|each_mod| {
    each_mod
      .depended_modules(module_graph)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(dep_mod_chunk_ref) = chunk_ref_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunk_ref[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_ref| *each_chunk_ref != dep_mod_chunk_ref)
            .for_each(|each_chunk_ref| {
              chunk_relation_graph2.add_edge(*each_chunk_ref, *dep_mod_chunk_ref, ());
            });
        }
      });
    each_mod
      .dynamic_depended_modules(module_graph)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(chunk_id) = chunk_ref_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunk_ref[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_id| *each_chunk_id != chunk_id)
            .for_each(|each_chunk_id| {
              chunk_relation_graph2.add_edge(*each_chunk_id, *chunk_id, ());
            });
        }
      });
  });

  // println!("chunk graph {:?}", Dot::new(&chunk_graph));

  for entry in &chunk_entries {
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_graph
        .module_by_uri(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);

        let belong_to_chunks = &mod_to_chunk_ref[module_uri];
        // println!(
        //   "[module {:?}]: belong to chunks {:?}",
        //   module_uri, belong_to_chunks
        // );
        belong_to_chunks
          .iter()
          .filter(|id_of_chunk_to_place_module| {
            if is_reuse_existing_chunk {
              // We only want to have chunks that have no superiors.
              // If both chunk A and B have the same module, we only want to place module into the uppermost chunk based on the relationship between A and B.
              let has_superior = belong_to_chunks.iter().any(|maybe_superior_chunk| {
                chunk_relation_graph2
                  .contains_edge(*maybe_superior_chunk, **id_of_chunk_to_place_module)
              });
              !has_superior
            } else {
              true
            }
          })
          .for_each(|id_of_chunk_to_place_module| {
            let chunk_to_place_module = chunk_by_ref
              .get_mut(id_of_chunk_to_place_module)
              .expect("Failed to get chunk by id");
            chunk_to_place_module
              .module_uris
              .insert(module_uri.to_string());
          });

        module
          .depended_modules(module_graph)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_splitting {
          module
            .dynamic_depended_modules(module_graph)
            .into_iter()
            .for_each(|module| queue.push_back(&module.uri));
        }
      } else {
        // TODO: detect circle import
      }
    }
  }

  // if bundle_options.optimization.remove_empty_chunks {
  if true {
    let empty_chunk_id_to_be_removed = chunk_by_ref
      .values()
      .filter(|chunk| chunk.module_uris.is_empty())
      .map(|chunk| chunk.rid)
      .collect::<Vec<_>>();

    empty_chunk_id_to_be_removed.iter().for_each(|chunk_ref| {
      chunk_by_ref.remove(chunk_ref);
    });
  }
}

struct ChunkIdGenerator<'me> {
  id_count: usize,
  chunk_id_algo: ChunkIdAlgo,
  module_graph: &'me ModuleGraph,
  root: &'me str,
}

impl<'me> ChunkIdGenerator<'me> {
  pub fn gen_id(&mut self, module_uri: &str) -> String {
    match self.chunk_id_algo {
      ChunkIdAlgo::Numeric => {
        let id = self.id_count.to_string();
        self.id_count += 1;
        id
      }
      ChunkIdAlgo::Named => {
        let js_mod = self
          .module_graph
          .module_by_uri(module_uri)
          .expect("Failed to get module by uri");
        js_mod
          .name
          .clone()
          .unwrap_or_else(|| uri_to_chunk_name(self.root, &js_mod.uri))
      }
    }
  }
}

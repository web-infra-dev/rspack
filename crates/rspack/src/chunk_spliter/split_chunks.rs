use std::collections::{HashMap, HashSet, VecDeque};

use rspack_core::{
  path::uri_to_chunk_name, BundleOptions, Chunk, ChunkGraph, ChunkIdAlgo, ChunkKind, JsModuleKind,
  ModuleGraphContainer,
};
use tracing::instrument;

#[instrument(skip_all)]
pub fn code_splitting2(
  module_graph_container: &ModuleGraphContainer,
  bundle_options: &BundleOptions,
) -> ChunkGraph {
  let code_splitting_options = &bundle_options.code_splitting;
  let is_enable_code_splitting = code_splitting_options.enable;
  let is_reuse_existing_chunk = code_splitting_options.reuse_existing_chunk;

  let mut chunk_graph = ChunkGraph::default();

  let mut id_generator = ChunkIdGenerator {
    id_count: 0,
    module_graph_container,
    bundle_options,
  };

  let module_graph = &module_graph_container.module_graph;

  let mut chunk_id_by_entry_module_uri = HashMap::new();
  let mut chunk_relation_graph2 = petgraph::graphmap::DiGraphMap::<&str, ()>::new();

  let mut chunk_entries = module_graph_container
    .resolved_entries
    .iter()
    .map(|(_name, rid)| rid.uri.as_str())
    .collect::<Vec<_>>();

  // First we need to create entry chunk.
  for entry in &chunk_entries {
    let chunk_id = id_generator.gen_id(*entry);
    let entry_module_name = module_graph
      .module_by_uri(*entry)
      .unwrap()
      .kind
      .name()
      .unwrap();
    let chunk = Chunk::new(
      chunk_id,
      Default::default(),
      entry.to_string(),
      ChunkKind::Entry {
        name: entry_module_name.to_string(),
      },
    );

    chunk_id_by_entry_module_uri.insert(*entry, chunk.id.clone());
    chunk_graph.add_chunk(chunk);
  }

  if is_enable_code_splitting {
    module_graph.modules().for_each(|module| {
      module
        .dynamic_dependency_modules(module_graph)
        .into_iter()
        .for_each(|dyn_dep_module| {
          chunk_id_by_entry_module_uri
            .entry(dyn_dep_module.uri.as_str())
            .or_insert_with_key(|mod_uri| {
              chunk_entries.push(*mod_uri);

              let chunk_id = id_generator.gen_id(*mod_uri);
              let chunk =
                Chunk::from_js_module(chunk_id.clone(), mod_uri.to_string(), ChunkKind::Normal);
              chunk_graph.add_chunk(chunk);
              chunk_id
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

  let mut mod_to_chunk_id: HashMap<&str, HashSet<&str>> = Default::default();

  for entry in &chunk_entries {
    let chunk_id = &chunk_id_by_entry_module_uri[*entry];
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_graph
        .module_by_uri(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);
        mod_to_chunk_id
          .entry(module_uri)
          .or_default()
          .insert(chunk_id.as_str());
        // let chunk = &mut chunk_graph[chunk_id];
        // chunk.module_ids.push(module_uri.to_string());
        module
          .dependency_modules(module_graph)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_splitting {
          module
            .dynamic_dependency_modules(module_graph)
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
      .dependency_modules(module_graph)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(dep_mod_chunk) = chunk_id_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunk_id[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_id| *each_chunk_id != dep_mod_chunk)
            .for_each(|each_chunk_id| {
              chunk_relation_graph2.add_edge(*each_chunk_id, dep_mod_chunk.as_str(), ());
            });
        }
      });
    each_mod
      .dynamic_dependency_modules(module_graph)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(chunk_id) = chunk_id_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunk_id[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_id| *each_chunk_id != chunk_id)
            .for_each(|each_chunk_id| {
              chunk_relation_graph2.add_edge(*each_chunk_id, chunk_id.as_str(), ());
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

        let belong_to_chunks = &mod_to_chunk_id[module_uri];
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
            let chunk_to_place_module = chunk_graph
              .chunk_by_id_mut(id_of_chunk_to_place_module)
              .unwrap();
            chunk_to_place_module
              .module_uris
              .insert(module_uri.to_string());
          });

        module
          .dependency_modules(module_graph)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_splitting {
          module
            .dynamic_dependency_modules(module_graph)
            .into_iter()
            .for_each(|module| queue.push_back(&module.uri));
        }
      } else {
        // TODO: detect circle import
      }
    }
  }

  if bundle_options.optimization.remove_empty_chunks {
    let empty_chunk_id_to_be_removed = chunk_graph
      .chunks()
      .filter(|chunk| chunk.module_uris.is_empty())
      .map(|chunk| chunk.id.clone())
      .collect::<Vec<_>>();

    empty_chunk_id_to_be_removed.iter().for_each(|chunk_id| {
      chunk_graph.remove_by_id(chunk_id);
    });
  }
  chunk_graph.chunks_mut().for_each(|chunk| {
    chunk.sort_modules(module_graph);
  });
  chunk_graph
}

#[instrument(skip_all)]
pub fn split_chunks(
  module_graph: &ModuleGraphContainer,
  bundle_options: &BundleOptions,
) -> ChunkGraph {
  code_splitting2(module_graph, bundle_options)
}

struct ChunkIdGenerator<'me> {
  id_count: usize,
  bundle_options: &'me BundleOptions,
  module_graph_container: &'me ModuleGraphContainer,
}

impl<'me> ChunkIdGenerator<'me> {
  pub fn gen_id(&mut self, module_uri: &str) -> String {
    match self.bundle_options.optimization.chunk_id_algo {
      ChunkIdAlgo::Numeric => {
        let id = self.id_count.to_string();
        self.id_count += 1;
        id
      }
      ChunkIdAlgo::Named => {
        let js_mod = self
          .module_graph_container
          .module_graph
          .module_by_uri(module_uri)
          .unwrap();
        if let JsModuleKind::UserEntry { name } = &js_mod.kind {
          name.to_string()
        } else {
          uri_to_chunk_name(&self.bundle_options.root, &js_mod.uri)
        }
      }
    }
  }
}

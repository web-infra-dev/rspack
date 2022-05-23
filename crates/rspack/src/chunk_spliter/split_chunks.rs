use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::graph::NodeIndex;
use rspack_core::{
  BundleOptions, Chunk, ChunkIdAlgo, ChunkKind, JsModule, JsModuleKind, ModuleGraph,
};
use tracing::instrument;

#[instrument]
pub fn code_splitting2(
  module_graph: &ModuleGraph,
  is_enable_code_spliting: bool,
  is_reuse_exsting_chunk: bool,
  chunk_id_algo: &ChunkIdAlgo,
) -> Vec<Chunk> {
  let mut id_count = 0;
  let mut gen_numeric_chunk_id = || {
    id_count += 1;
    format!("{:?}", id_count)
  };

  let module_by_uri: &HashMap<String, JsModule> = &module_graph.module_by_id;

  let mut chunk_id_by_entry_module_uri = HashMap::new();
  let mut chunk_graph = petgraph::Graph::<Chunk, ()>::new();

  let mut chunk_entries = module_graph
    .resolved_entries
    .iter()
    .map(|(_name, rid)| rid.uri.as_str())
    .collect::<Vec<_>>();

  // First we need to create entry chunk.
  for entry in &chunk_entries {
    let name = {
      let js_mod = module_by_uri.get(*entry).unwrap();
      if let JsModuleKind::UserEntry { name } = &js_mod.kind {
        name.to_string()
      } else {
        format!("path {:?}", gen_numeric_chunk_id())
      }
    };
    let chunk_id = {
      match chunk_id_algo {
        ChunkIdAlgo::Named => name,
        ChunkIdAlgo::Numeric => gen_numeric_chunk_id(),
      }
    };
    let entry_module_name = module_by_uri.get(*entry).unwrap().kind.name().unwrap();
    let mut chunk = Chunk::new(
      chunk_id,
      Default::default(),
      entry.to_string(),
      ChunkKind::Entry {
        name: entry_module_name.to_string(),
      },
    );

    let chunk_grpah_id = chunk_graph.add_node(chunk);
    chunk_id_by_entry_module_uri.insert(*entry, chunk_grpah_id);
  }
  if is_enable_code_spliting {
    module_by_uri.values().for_each(|module| {
      module
        .dynamic_dependency_modules(module_by_uri)
        .into_iter()
        .for_each(|dyn_dep_module| {
          chunk_id_by_entry_module_uri
            .entry(dyn_dep_module.uri.as_str())
            .or_insert_with_key(|mod_uri| {
              chunk_entries.push(*mod_uri);
              let name = {
                let js_mod = module_by_uri.get(*mod_uri).unwrap();
                if let JsModuleKind::UserEntry { name } = &js_mod.kind {
                  name.to_string()
                } else {
                  format!("path {:?}", gen_numeric_chunk_id())
                }
              };
              let chunk_id = {
                match chunk_id_algo {
                  ChunkIdAlgo::Named => name,
                  ChunkIdAlgo::Numeric => gen_numeric_chunk_id(),
                }
              };
              let mut chunk =
                Chunk::from_js_module(chunk_id, dyn_dep_module.uri.to_string(), ChunkKind::Normal);

              chunk_graph.add_node(chunk)
            });
        });
    });
  }

  // Now, we have all chunks and need place right modules into chunks.

  let mut mod_to_chunks: HashMap<&str, HashSet<NodeIndex>> = Default::default();

  for entry in &chunk_entries {
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_by_uri
        .get(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);
        let chunk_id = chunk_id_by_entry_module_uri[*entry];
        mod_to_chunks
          .entry(module_uri)
          .or_default()
          .insert(chunk_id);
        // let chunk = &mut chunk_graph[chunk_id];
        // chunk.module_ids.push(module_uri.to_string());
        module
          .dependency_modules(module_by_uri)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_spliting {
          module
            .dynamic_dependency_modules(module_by_uri)
            .into_iter()
            .for_each(|module| queue.push_back(&module.uri));
        }
      } else {
        // TODO: detect circle import
      }
    }
  }

  module_by_uri.values().for_each(|each_mod| {
    each_mod
      .dependency_modules(module_by_uri)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(dep_mod_chunk) = chunk_id_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunks[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_id| *each_chunk_id != dep_mod_chunk)
            .for_each(|each_chunk_id| {
              chunk_graph.add_edge(*each_chunk_id, *dep_mod_chunk, ());
            });
        }
      });
    each_mod
      .dynamic_dependency_modules(module_by_uri)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(chunk_id) = chunk_id_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunks[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_id| *each_chunk_id != chunk_id)
            .for_each(|each_chunk_id| {
              chunk_graph.add_edge(*each_chunk_id, *chunk_id, ());
            });
        }
      });
  });

  // println!("chunk graph {:?}", Dot::new(&chunk_graph));

  for entry in &chunk_entries {
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_by_uri
        .get(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);

        let belong_to_chunks = &mod_to_chunks[module_uri];
        // println!(
        //   "[module {:?}]: belong to chunks {:?}",
        //   module_uri, belong_to_chunks
        // );
        belong_to_chunks
          .iter()
          .filter(|id_of_chunk_to_place_module| {
            if is_reuse_exsting_chunk {
              // We only want to have chunks the hash no superiors.
              let has_superior = belong_to_chunks.iter().any(|maybe_superior_chunk| {
                chunk_graph.contains_edge(*maybe_superior_chunk, **id_of_chunk_to_place_module)
              });
              !has_superior
            } else {
              true
            }
          })
          .collect::<Vec<_>>()
          .into_iter()
          .for_each(|id_of_chunk_to_place_module| {
            let chunk_to_place_module = &mut chunk_graph[*id_of_chunk_to_place_module];
            chunk_to_place_module
              .module_ids
              .insert(module_uri.to_string());
          });

        module
          .dependency_modules(module_by_uri)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_spliting {
          module
            .dynamic_dependency_modules(module_by_uri)
            .into_iter()
            .for_each(|module| queue.push_back(&module.uri));
        }
      } else {
        // TODO: detect circle import
      }
    }
  }

  let (chunks, _) = chunk_graph.into_nodes_edges();
  chunks
    .into_iter()
    .map(|node| node.weight)
    .collect::<Vec<_>>()
}

#[instrument(skip(module_graph))]
pub fn split_chunks(module_graph: &ModuleGraph, bundle_options: &BundleOptions) -> Vec<Chunk> {
  let code_splitting_options = &bundle_options.code_splitting;
  let is_enable_code_spliting;
  let is_reuse_exsting_chunk;
  if let Some(option) = code_splitting_options {
    is_enable_code_spliting = true;
    is_reuse_exsting_chunk = option.reuse_existing_chunk;
  } else {
    is_enable_code_spliting = false;
    is_reuse_exsting_chunk = false;
  }
  code_splitting2(
    module_graph,
    is_enable_code_spliting,
    is_reuse_exsting_chunk,
    &bundle_options.optimization.chunk_id_algo,
  )
}

use std::collections::{HashMap, HashSet, VecDeque};

use rspack_core::{Chunk, JsModule, ModuleGraph};
use tracing::instrument;

#[instrument]
pub fn code_splitting2(module_graph: &ModuleGraph, is_enable_code_spliting: bool) -> Vec<Chunk> {
  let mut id_count = 0;

  let module_by_uri: &HashMap<String, JsModule> = &module_graph.module_by_id;
  // let dependency_graph_container = DependencyGraph::from_modules(module_by_id);
  // let dependency_graph = dependency_graph_container.graph();

  let mut chunk_id_by_entry_module_id = HashMap::new();
  let mut chunk_graph = petgraph::Graph::<Chunk, i32>::new();

  let mut entries = module_graph
    .resolved_entries
    .iter()
    .map(|rid| rid.uri.as_str())
    .collect::<Vec<_>>();

  // First we need to create entry chunk.
  for entry in &entries {
    let mut chunk = Chunk::new(vec![entry.to_string().into()], entry.to_string(), true);
    chunk.id = {
      id_count += 1;
      format!("{:?}", id_count)
    };
    let chunk_id = chunk_graph.add_node(chunk);
    chunk_id_by_entry_module_id.insert(*entry, chunk_id);
  }
  if is_enable_code_spliting {
    module_by_uri.values().for_each(|module| {
      module
        .dynamic_dependency_modules(module_by_uri)
        .into_iter()
        .for_each(|dyn_dep_module| {
          chunk_id_by_entry_module_id
            .entry(dyn_dep_module.uri.as_str())
            .or_insert_with_key(|id| {
              entries.push(*id);
              let mut chunk = Chunk::from_js_module(dyn_dep_module.uri.to_string().into(), false);
              chunk.id = {
                id_count += 1;
                format!("{:?}", id_count)
              };
              let chunk_id = chunk_graph.add_node(chunk);
              chunk_id
            });
        });
    });
  }

  for entry in &entries {
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_by_uri
        .get(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);
        let chunk = &mut chunk_graph[chunk_id_by_entry_module_id[*entry]];
        chunk.module_ids.push(module_uri.to_string());
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
pub fn split_chunks(module_graph: &ModuleGraph, is_enable_code_spliting: bool) -> Vec<Chunk> {
  code_splitting2(module_graph, is_enable_code_spliting)
}

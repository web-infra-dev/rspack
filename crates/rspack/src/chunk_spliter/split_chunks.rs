use std::collections::{HashMap, HashSet, LinkedList};

use petgraph::{
  graph::NodeIndex,
  visit::{depth_first_search, Control, DfsEvent},
  EdgeDirection,
};
use rspack_core::{Chunk, JsModule, ModuleGraph, ResolvedURI};
use tracing::instrument;

#[derive(Clone, Debug)]
struct Dependency {
  is_async: bool,
}

type ModulePetGraph<'a> = petgraph::graphmap::DiGraphMap<&'a str, Dependency>;

#[instrument(skip(module_graph))]
pub fn split_chunks(module_graph: &ModuleGraph, is_enable_code_spliting: bool) -> Vec<Chunk> {
  let module_by_id: &HashMap<String, JsModule> = &module_graph.module_by_id;
  let resolved_entries: &Vec<ResolvedURI> = &module_graph.resolved_entries;
  let mut dependency_graph = ModulePetGraph::new();
  module_by_id.keys().for_each(|module_id| {
    dependency_graph.add_node(module_id);
  });

  let mut edges: Vec<(String, String, Dependency)> = vec![];
  module_by_id.values().for_each(|module| {
    // let module = &self.graph.module_by_id[module_id];

    module
      .dependencies
      .keys()
      .collect::<Vec<_>>()
      .into_iter()
      .for_each(|dep| {
        let dep_uri = module.resolved_uris.get(dep).unwrap().clone();
        edges.push((
          module.uri.clone(),
          dep_uri.uri,
          Dependency { is_async: false },
        ))
      });
    module
      .dyn_imports
      .iter()
      .collect::<Vec<_>>()
      .into_iter()
      .for_each(|dep| {
        let dep_rid = module.resolved_uris.get(&dep.argument).unwrap().clone();
        edges.push((
          module.uri.clone(),
          dep_rid.uri,
          Dependency { is_async: true },
        ))
      });
  });
  edges.iter().for_each(|(from, to, edge)| {
    dependency_graph.add_edge(from, to, edge.clone());
  });

  let mut chunk_roots = HashMap::new();
  let mut reachable_chunks = HashSet::new();
  let mut chunk_graph = petgraph::Graph::<Chunk, i32>::new();

  let entries = resolved_entries
    .iter()
    .map(|rid| rid.uri.as_str())
    .collect::<Vec<_>>();

  for entry in &entries {
    let chunk = Chunk::new(vec![entry.to_string().into()], entry.to_string(), true);
    let chunk_id = chunk_graph.add_node(chunk);
    chunk_roots.insert(*entry, (chunk_id, chunk_id));
  }

  let mut stack = LinkedList::new();
  depth_first_search(&dependency_graph, entries.clone(), |event| {
    match event {
      DfsEvent::Discover(module_idx, _) => {
        // Push to the stack when a new chunk is created.
        if let Some((_, chunk_group_id)) = chunk_roots.get(&module_idx) {
          stack.push_front((module_idx, *chunk_group_id));
        }
      }
      DfsEvent::TreeEdge(importer_id, importee_id) => {
        // Create a new chunk if the dependency is async.
        let dependency = &dependency_graph[(importer_id, importee_id)];
        if dependency.is_async && is_enable_code_spliting {
          let chunk = Chunk::from_js_module(importee_id.to_string().into(), false);
          let chunk_id = chunk_graph.add_node(chunk);
          chunk_roots.insert(importee_id, (chunk_id, chunk_id));

          // Walk up the stack until we hit a different asset type
          // and mark each this bundle as reachable from every parent bundle.
          for (chunk_entry_module_idx, _) in &stack {
            reachable_chunks.insert((*chunk_entry_module_idx, importee_id));
          }
        }
      }
      DfsEvent::Finish(finished_module_id, _) => {
        // Pop the stack when existing the asset node that created a bundle.
        if let Some((module_id, _)) = stack.front() {
          if *module_id == finished_module_id {
            stack.pop_front();
          }
        }
      }
      _ => {}
    }
  });

  let mut entries = HashSet::new();
  resolved_entries.iter().for_each(|entry| {
    entries.insert(entry.uri.clone());
  });

  let mut reachable_modules = HashSet::new();

  for root_which_is_node_idx_of_chunks_entry_module in chunk_roots.keys() {
    depth_first_search(
      &dependency_graph,
      Some(*root_which_is_node_idx_of_chunks_entry_module),
      |event| {
        if let DfsEvent::Discover(node_idx_of_visiting_module, _) = &event {
          if node_idx_of_visiting_module == root_which_is_node_idx_of_chunks_entry_module {
            return Control::Continue;
          }
          reachable_modules.insert((
            *root_which_is_node_idx_of_chunks_entry_module,
            *node_idx_of_visiting_module,
          ));

          // Stop when we hit another bundle root.
          if chunk_roots.contains_key(*node_idx_of_visiting_module) {
            return Control::<()>::Prune;
          }
        }
        Control::Continue
      },
    );
  }

  let reachable_module_graph =
    petgraph::graphmap::DiGraphMap::<&'_ str, ()>::from_edges(&reachable_modules);

  // Step 3: Place all modules into chunks. Each module is placed into a single
  // chunk based on the chunk entries it is reachable from. This creates a
  // maximally code split chunk graph with no duplication.

  // Create a mapping from entry module ids to chunk ids.
  let mut chunks: HashMap<Vec<&str>, NodeIndex> = HashMap::new();
  let mut module_ids = dependency_graph.nodes().collect::<Vec<_>>();
  module_ids.sort_by_key(|module_id| chunk_roots.contains_key(*module_id));
  module_ids.reverse();
  for module_id in module_ids {
    // Find chunk entries reachable from the module.
    let reachable: Vec<&str> = reachable_module_graph
      .neighbors_directed(module_id, EdgeDirection::Incoming)
      .collect();
    // Filter out chunks when the module is reachable in a parent chunk.
    let reachable: Vec<&str> = reachable
      .iter()
      .cloned()
      .filter(|b| {
        reachable
          .iter()
          .all(|a| !reachable_chunks.contains(&(*a, *b)))
      })
      .collect();

    if let Some((chunk_id, _)) = chunk_roots.get(&module_id) {
      // If the module is a chunk root, add the chunk to every other reachable chunk group.
      chunks.entry(vec![module_id]).or_insert(*chunk_id);
      for a in &reachable {
        if *a != module_id {
          chunk_graph.add_edge(chunk_roots[a].1, *chunk_id, 0);
        }
      }
    } else if !reachable.is_empty() {
      // If the asset is reachable from more than one entry, find or create
      // a chunk for that combination of entries, and add the asset to it.
      let source_chunks = reachable
        .iter()
        .map(|a| chunks[&vec![*a]])
        .collect::<Vec<_>>();
      let chunk_id = chunks.entry(reachable.clone()).or_insert_with(|| {
        let mut chunk = Chunk::default();
        chunk.source_chunks = source_chunks;
        chunk_graph.add_node(chunk)
      });

      let bundle = &mut chunk_graph[*chunk_id];
      if bundle.entry.is_empty() {
        bundle.entry = module_id.to_string().into();
      }
      bundle.module_ids.push(module_id.to_string().into());
      // bundle.size += module_by_id[module_id].size;

      // Add the bundle to each reachable bundle group.
      for item_module_id in reachable {
        let item_chunk_id = chunk_roots[&item_module_id].1;
        if item_chunk_id != *chunk_id {
          chunk_graph.add_edge(item_chunk_id, *chunk_id, 0);
        }
      }
    }
  }

  for chunk_id in chunk_graph.node_indices() {
    let chunk = &chunk_graph[chunk_id];
    if !is_enable_code_spliting && chunk.source_chunks.len() > 0 {
      remove_chunk(&dependency_graph, &mut chunk_graph, chunk_id);
    }
  }

  let (chunks, _) = chunk_graph.into_nodes_edges();

  chunks
    .into_iter()
    .map(|node| node.weight)
    .collect::<Vec<_>>()
}

fn remove_chunk(
  _dep_graph: &ModulePetGraph,
  chunk_graph: &mut petgraph::graph::Graph<Chunk, i32>,
  chunk_id: NodeIndex,
) {
  let bundle = chunk_graph.remove_node(chunk_id).unwrap();
  for asset_id in &bundle.module_ids {
    for source_bundle_id in &bundle.source_chunks {
      let bundle = &mut chunk_graph[*source_bundle_id];
      bundle.module_ids.push(asset_id.clone());
    }
  }
}

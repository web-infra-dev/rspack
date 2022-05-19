use std::collections::{HashMap, HashSet, LinkedList};

use petgraph::{
  graph::NodeIndex,
  visit::{depth_first_search, Control, DfsEvent},
  EdgeDirection,
};
use rspack_core::{Chunk, JsModule, ModuleGraph, ResolvedURI};
use tracing::instrument;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Dependency {
  is_async: bool,
}

type ModulePetGraph<'a> = petgraph::graphmap::DiGraphMap<&'a str, Dependency>;

struct DependencyGraph {
  edges: HashSet<(String, String, Dependency)>,
  // graph: ModulePetGraph<'me>,
  // _phantom: PhantomData<'me>,
}

impl DependencyGraph {
  pub fn from_modules(module_by_id: &HashMap<String, JsModule>) -> Self {
    // TODO: we need to consider about an module could be both static and dynamic imported.
    let mut edge_by_weight = HashMap::new();
    module_by_id
      .values()
      .flat_map(|module| {
        module
          .dependencies
          .keys()
          // .collect::<Vec<_>>()
          .into_iter()
          .map(|dep| {
            let dep_uri = module.resolved_uris.get(dep).unwrap().clone();
            (
              module.uri.clone(),
              dep_uri.uri,
              Dependency { is_async: false },
            )
          })
          .chain(
            module
              .dyn_imports
              .iter()
              .collect::<Vec<_>>()
              .into_iter()
              .map(|dep| {
                let dep_rid = module.resolved_uris.get(&dep.argument).unwrap().clone();
                (
                  module.uri.clone(),
                  dep_rid.uri,
                  Dependency { is_async: true },
                )
              }),
          )
      })
      .for_each(|(from, to, weight)| {
        edge_by_weight.entry((from, to)).or_insert(weight);
      });

    DependencyGraph {
      edges: edge_by_weight
        .into_iter()
        .map(|((from, to), weight)| (from, to, weight))
        .collect(),
    }
  }

  pub fn graph(&self) -> ModulePetGraph<'_> {
    let dependency_graph = ModulePetGraph::from_edges(
      self
        .edges
        .iter()
        .map(|(from, to, weight)| (from.as_str(), to.as_str(), weight.clone())),
    );

    dependency_graph
  }
}

// Only split code imported dynamically into chunks
// Chunks may contains duplicate code, but it is not a problem in runtime.
#[instrument(skip_all)]
pub fn code_splitting(module_graph: &ModuleGraph, is_enable_code_spliting: bool) -> Vec<Chunk> {
  let module_by_id: &HashMap<String, JsModule> = &module_graph.module_by_id;
  let dependency_graph_container = DependencyGraph::from_modules(module_by_id);
  let dependency_graph = dependency_graph_container.graph();

  let mut chunk_id_by_entry_module_id = HashMap::new();
  // reachable_chunks
  //
  let mut reachable_chunks = HashSet::new();
  let mut chunk_graph = petgraph::Graph::<Chunk, i32>::new();

  let entries = module_graph
    .resolved_entries
    .iter()
    .map(|rid| rid.uri.as_str())
    .collect::<Vec<_>>();

  // First we need to create entry chunk.
  for entry in &entries {
    let chunk = Chunk::new(vec![entry.to_string().into()], entry.to_string(), true);
    let chunk_id = chunk_graph.add_node(chunk);
    chunk_id_by_entry_module_id.insert(*entry, chunk_id);
  }

  let mut stack = LinkedList::new();
  depth_first_search(&dependency_graph, entries.clone(), |event| {
    match event {
      DfsEvent::Discover(module_idx, _) => {
        // Push to the stack when a new chunk is created.
        if let Some(chunk_id) = chunk_id_by_entry_module_id.get(&module_idx) {
          stack.push_front((module_idx, *chunk_id));
        }
      }
      DfsEvent::TreeEdge(importer_id, importee_id) => {
        // Create a new chunk if the dependency is async.
        let dependency = &dependency_graph[(importer_id, importee_id)];
        if dependency.is_async && is_enable_code_spliting {
          println!(
            "create chunk due to dynamic import {:?} by {}",
            importee_id, importer_id
          );
          let chunk = Chunk::from_js_module(importee_id.to_string().into(), false);
          let chunk_id = chunk_graph.add_node(chunk);
          chunk_id_by_entry_module_id.insert(importee_id, chunk_id);

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

  let mut reachable_modules = HashSet::new();

  for entry_module_id in chunk_id_by_entry_module_id.keys() {
    depth_first_search(&dependency_graph, Some(*entry_module_id), |event| {
      if let DfsEvent::Discover(visiting_module_idx, _) = &event {
        if visiting_module_idx == entry_module_id {
          return Control::Continue;
        }
        reachable_modules.insert((*entry_module_id, *visiting_module_idx));

        // Stop when we hit another bundle root.
        if chunk_id_by_entry_module_id.contains_key(*visiting_module_idx) {
          return Control::<()>::Prune;
        }
      }
      Control::Continue
    });
  }

  let reachable_module_graph =
    petgraph::graphmap::DiGraphMap::<&'_ str, ()>::from_edges(&reachable_modules);

  // Step 3: Place all modules into chunks. Each module is placed into a single
  // chunk based on the chunk entries it is reachable from. This creates a
  // maximally code split chunk graph with no duplication.

  // Create a mapping from entry module ids to chunk ids.
  // let mut chunks: HashMap<Vec<&str>, NodeIndex> = HashMap::new();
  let mut module_ids = dependency_graph.nodes().collect::<Vec<_>>();
  // module_ids.sort_by_key(|module_id| chunk_id_by_entry_module_id.contains_key(*module_id));
  // module_ids.reverse();
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

    if let Some(chunk_id) = chunk_id_by_entry_module_id.get(&module_id) {
      // If the module is a chunk root, add the chunk to every other reachable chunk group.
      // chunks.entry(vec![module_id]).or_insert(*chunk_id);
      for a in &reachable {
        if *a != module_id {
          chunk_graph.add_edge(chunk_id_by_entry_module_id[a], *chunk_id, 0);
        }
      }
    } else if !reachable.is_empty() {
      // If the asset is reachable from more than one entry, find or create
      // a chunk for that combination of entries, and add the asset to it.
      let source_chunks = reachable
        .iter()
        .map(|a| chunk_id_by_entry_module_id[*a])
        .collect::<Vec<_>>();
      source_chunks.iter().for_each(|chunk_id| {
        let chunk = &mut chunk_graph[*chunk_id];
        chunk.module_ids.push(module_id.to_string().into());
      });
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
  code_splitting(module_graph, is_enable_code_spliting)
}

use std::{
    collections::{HashMap, HashSet, LinkedList},
    sync::{Arc, Mutex},
};

use dashmap::DashSet;
use petgraph::{
    dot::Dot,
    visit::{depth_first_search, DfsEvent},
};
use smol_str::SmolStr;

use crate::{
    bundler::BundleOptions, chunk::Chunk, mark_box::MarkBox, module_graph::ModuleGraph,
    structs::OutputChunk,
};

#[non_exhaustive]
pub struct Bundle {
    pub graph: ModuleGraph,
    pub output_options: Arc<BundleOptions>,
    pub mark_box: Arc<Mutex<MarkBox>>,
}

#[derive(Clone, Debug)]
struct Dependency {
    is_async: bool,
}
type ModulePetGraph<'a> = petgraph::graphmap::DiGraphMap<&'a str, Dependency>;

impl Bundle {
    pub fn new(
        graph: ModuleGraph,
        output_options: Arc<BundleOptions>,
        mark_box: Arc<Mutex<MarkBox>>,
    ) -> Self {
        Self {
            graph,
            output_options,
            mark_box,
        }
    }

    fn generate_chunks(&self) -> Vec<Chunk> {
        let mut dependency_graph = ModulePetGraph::new();
        self.graph
            .ordered_modules
            .iter()
            .rev()
            .for_each(|module_id| {
                dependency_graph.add_node(module_id);
            });

        let mut egdes = vec![];
        self.graph.ordered_modules.iter().for_each(|module_id| {
            let module = &self.graph.module_by_id[module_id];

            module
                .dependencies
                .keys()
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|dep| {
                    let dep_rid = module.resolved_ids().get(dep).unwrap().clone();
                    egdes.push((
                        module_id.clone(),
                        dep_rid.id.clone(),
                        Dependency { is_async: false },
                    ))
                });
            module
                .dyn_dependencies
                .iter()
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|dep| {
                    let dep_rid = module.resolved_ids().get(&dep.argument).unwrap().clone();
                    egdes.push((
                        module_id.clone(),
                        dep_rid.id.clone(),
                        Dependency { is_async: true },
                    ))
                });
        });
        egdes.iter().for_each(|(from, to, edge)| {
            dependency_graph.add_edge(from, to, edge.clone());
        });

        let mut chunk_roots = HashMap::new();
        // let mut reachable_chunks = HashSet::new();
        let mut chunk_graph = petgraph::Graph::<Chunk, i32>::new();

        println!("dep graph {:#?}", Dot::new(&dependency_graph));
        let entries = self
            .graph
            .resolved_entries
            .iter()
            .map(|rid| rid.id.clone())
            .collect::<Vec<_>>();
        for entry in &entries {
            let chunk_id = chunk_graph.add_node(Chunk {
                id: Default::default(),
                module_ids: vec![entry.clone()],
                entries: entry.clone(),
            });
            chunk_roots.insert(entry.as_str(), (chunk_id, chunk_id));
        }

        // let mut stack = LinkedList::new();
        // depth_first_search(
        //     &dependency_graph,
        //     &entries.iter().map(|id| id.as_str()).collect(),
        //     |event| {
        //         match event {
        //             DfsEvent::Discover(module_idx, _) => {
        //                 // println!("Discover {:?}", module_idx);
        //                 // Push to the stack when a new chunk is created.
        //                 if let Some((_, chunk_group_id)) = chunk_roots.get(&module_idx) {
        //                     // stack 的队头表示的 chunk 入口模块的 图索引 和其所属的 chunk 的 id
        //                     stack.push_front((module_idx, *chunk_group_id));
        //                 }
        //             }
        //             DfsEvent::TreeEdge(importer_id, importee_id) => {
        //                 // println!("TreeEdge from {:?} to {:?}", importer_id, importee_id);
        //                 // Create a new bundle as well as a new bundle group if the dependency is async.

        //                 let dependency = &dependency_graph[(importer_id, importee_id)];
        //                 if dependency.is_async {
        //                     let chunk = Chunk::from_js_module(importee_id.to_string().into());
        //                     let chunk_id = chunk_graph.add_node(chunk);
        //                     chunk_roots.insert(importee_id, (chunk_id, chunk_id));

        //                     // Walk up the stack until we hit a different asset type
        //                     // and mark each this bundle as reachable from every parent bundle.
        //                     for (chunk_entry_module_idx, _) in &stack {
        //                         reachable_chunks.insert((*chunk_entry_module_idx, importee_id));
        //                     }
        //                 }
        //             }
        //             DfsEvent::Finish(finished_module_id, _) => {
        //                 // println!("Finish {:?}", finished_module_id);
        //                 // Pop the stack when existing the asset node that created a bundle.
        //                 if let Some((module_id, _)) = stack.front() {
        //                     if *module_id == finished_module_id {
        //                         stack.pop_front();
        //                     }
        //                 }
        //             }
        //             _ => {}
        //         }
        //     },
        // );

        // TODO: code spliting
        let entries = DashSet::new();
        self.graph.resolved_entries.iter().for_each(|entry| {
            entries.insert(entry.id.clone());
        });

        let chunks = vec![];
        // let chunks = vec![Chunk {
        //     id: Default::default(),
        //     order_modules: self.graph.ordered_modules.clone(),
        //     entries,
        // }];

        chunks
    }

    pub fn generate(&mut self) -> HashMap<String, OutputChunk> {
        let mut chunks = self.generate_chunks();

        chunks.iter_mut().for_each(|chunk| {
            chunk.id = chunk.generate_id(&self.output_options);
        });

        chunks
            .iter_mut()
            .map(|chunk| {
                let chunk = chunk.render(&self.output_options, &mut self.graph.module_by_id);
                (
                    chunk.file_name.clone(),
                    OutputChunk {
                        code: chunk.code,
                        file_name: chunk.file_name,
                    },
                )
            })
            .collect()
    }
}

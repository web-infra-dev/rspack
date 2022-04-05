use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crossbeam::channel::{self};
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use petgraph::{
    dot::Dot,
    graph::NodeIndex,
    visit::{depth_first_search, DfsEvent},
};
use smol_str::SmolStr;

use crate::{
    chunk::Chunk, js_ext_module::JsExtModule, js_module::JsModule, plugin::ResolvedId,
    plugin_driver::PluginDriver, worker::Worker,
};

#[derive(Debug, Clone, Copy)]
pub enum Relation {
    AsyncImport,
    StaticImport,
}

type ModuleGraph = petgraph::Graph<SmolStr, Relation>;
type ChunkGraph = petgraph::Graph<SmolStr, ()>;

pub enum Msg {
    DependencyReference(SmolStr, SmolStr, Relation),
    NewMod(JsModule),
    NewExtMod(JsExtModule),
}

#[derive(Debug)]
pub struct InputOptions {
    pub entries: Vec<String>,
}

pub struct GraphContainer {
    pub plugin_driver: Arc<PluginDriver>,
    pub resolved_entries: Vec<ResolvedId>,
    pub module_by_id: HashMap<SmolStr, JsModule>,
    pub input: InputOptions,
}

impl GraphContainer {
    // build dependency graph via entry modules.
    pub async fn generate_module_graph(&mut self) {
        let nums_of_thread = num_cpus::get();
        let idle_thread_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(nums_of_thread));
        let job_queue: Arc<SegQueue<ResolvedId>> = Default::default();

        self.resolved_entries = join_all(
            self.input
                .entries
                .iter()
                .map(|entry| self.plugin_driver.resolve_id(None, entry)),
        )
        .await
        .into_iter()
        .collect();

        println!("resolved_entries {:?}", self.resolved_entries);

        let mut path_to_node_idx: HashMap<SmolStr, NodeIndex> = Default::default();
        let mut module_graph = ModuleGraph::new();

        self.resolved_entries.iter().for_each(|resolved_entry_id| {
            let entry_idx = module_graph.add_node(resolved_entry_id.id.clone());
            // self.entry_indexs.push(entry_idx);
            path_to_node_idx.insert(resolved_entry_id.id.clone(), entry_idx);
            job_queue.push(resolved_entry_id.clone());
        });

        let processed_id: Arc<DashSet<SmolStr>> = Default::default();
        let (tx, rx) = channel::unbounded::<Msg>();
        println!("job_queue {:?}", job_queue.len());
        for idx in 0..nums_of_thread {
            println!("spawing {:?}", idx);
            let idle_thread_count = idle_thread_count.clone();
            let plugin_driver = self.plugin_driver.clone();
            let worker = Worker {
                tx: tx.clone(),
                job_queue: job_queue.clone(),
                processed_id: processed_id.clone(),
                plugin_driver,
            };
            tokio::task::spawn(async move {
                'root: loop {
                    println!("worker: {:?}", idx);
                    idle_thread_count.fetch_sub(1, Ordering::SeqCst);
                    worker.run().await;
                    idle_thread_count.fetch_add(1, Ordering::SeqCst);
                    loop {
                        if !worker.job_queue.is_empty() {
                            break;
                            // need to work again
                        } else if idle_thread_count.load(Ordering::SeqCst) == nums_of_thread {
                            // All threads are idle now. There's no more work to do.
                            break 'root;
                        }
                    }
                }
            });
        }

        while idle_thread_count.load(Ordering::SeqCst) != nums_of_thread
            || job_queue.len() > 0
            || !rx.is_empty()
        {
            if let Ok(job) = rx.try_recv() {
                match job {
                    Msg::NewMod(module) => {
                        self.module_by_id.insert(module.id.clone(), module);
                    }
                    Msg::DependencyReference(from, to, rel) => {
                        let from_id = *path_to_node_idx
                            .entry(from)
                            .or_insert_with_key(|key| module_graph.add_node(key.clone()));
                        let to_id = *path_to_node_idx
                            .entry(to)
                            .or_insert_with_key(|key| module_graph.add_node(key.clone()));
                        module_graph.add_edge(from_id, to_id, rel);
                    }
                    _ => {}
                }
            }
        }
        println!("graph: {:?}", Dot::new(&module_graph));

        // Every chunk has a entry module. We use entry module id to represent a chunk.
        let mut chunks_by_entry_module_id: HashMap<SmolStr, Chunk> = HashMap::new();
        let mut module_id_to_its_chunk: HashMap<SmolStr, SmolStr> = HashMap::new();
        self.resolved_entries.iter().for_each(|resolved_id| {
            let entrt_module_id = &resolved_id.id;
            let chunk = Chunk {
                // id: resolved_id.id.clone(),
                module_ids: vec![entrt_module_id.clone()],
            };
            chunks_by_entry_module_id.insert(entrt_module_id.clone(), chunk);
            module_id_to_its_chunk.insert(entrt_module_id.clone(), entrt_module_id.clone());
        });
        let entries_node_idx = self
            .resolved_entries
            .iter()
            .map(|rid| path_to_node_idx[&rid.id])
            .collect::<Vec<_>>();
        type ChunkGraph = petgraph::graph::Graph<SmolStr, Relation>;
        let mut chunk_graph = ChunkGraph::new();
        let mut stack = vec![];
        depth_first_search(&module_graph, entries_node_idx, |evt| match evt {
            DfsEvent::Discover(module_node_idx, _) => {
                let module_id = &module_graph[module_node_idx];
                if let Some(_) = chunks_by_entry_module_id.get(module_id) {
                    stack.push(module_id.clone());
                } else {
                  let chunk = chunks_by_entry_module_id.get_mut(stack.last().unwrap()).unwrap();
                  chunk.module_ids.push(module_id.clone());
                }
            }
            DfsEvent::TreeEdge(from, to) => {
              let importer = &module_graph[from];
              let importee = &module_graph[to];
              let dependency = &module_graph[module_graph.find_edge(from, to).unwrap()];
              if let Relation::AsyncImport = dependency {
                
              }

            }
            _ => {}
        });
    }

    pub fn sort_modules(&self) -> Vec<SmolStr> {
        vec![]
    }
}

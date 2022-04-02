use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, collections::HashMap};

use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use petgraph::{graph::NodeIndex, dot::Dot};
use smol_str::SmolStr;
use crossbeam::channel::{self};


use crate::{plugin_driver::PluginDriver, plugin::ResolvedId, js_module::JsModule, DepGraph, Msg, worker::Worker};


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
        let mut dep_graph = DepGraph::new();

        self.resolved_entries.iter().for_each(|resolved_entry_id| {
            let entry_idx = dep_graph.add_node(resolved_entry_id.id.clone());
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
                            .or_insert_with_key(|key| dep_graph.add_node(key.clone()));
                        let to_id = *path_to_node_idx
                            .entry(to)
                            .or_insert_with_key(|key| dep_graph.add_node(key.clone()));
                        dep_graph.add_edge(from_id, to_id, rel);
                    }
                    _ => {}
                }
            }
        }

        // println!("grpah {:?}", Dot::new(&dep_graph))

        // let entries_id = self
        //     .entry_indexs
        //     .iter()
        //     .map(|idx| &self.module_graph[*idx])
        //     .collect::<HashSet<&SmolStr>>();
        // self.module_by_id.par_iter_mut().for_each(|(_key, module)| {
        //     module.is_user_defined_entry_point = entries_id.contains(&module.id);
        // });
    }
}

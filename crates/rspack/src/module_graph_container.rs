use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use crossbeam::{
    channel::{self},
    queue::SegQueue,
};
use dashmap::{DashSet};
use futures::future::join_all;
use petgraph::{graph::NodeIndex, visit::EdgeRef, EdgeDirection};
use rayon::prelude::*;
use smol_str::SmolStr;



use crate::{
    bundler::BundleOptions, external_module::ExternalModule, module::Module,
    plugin_driver::PluginDriver, scanner::rel::RelationInfo, mark_box::MarkBox,
    structs::ResolvedId, utils::resolve_id, worker::Worker, module_graph::ModuleGraph,
};

type ModulePetGraph = petgraph::graph::DiGraph<SmolStr, Rel>;

pub struct ModuleGraphContainer {
  resolved_entries: Vec<ResolvedId>,
  pub path_to_node_idx: HashMap<SmolStr, NodeIndex>,
  pub relation_graph: ModulePetGraph,

  pub bundle_options: Arc<BundleOptions>,
  pub entry_indexs: Vec<NodeIndex>,
  pub ordered_modules: Vec<NodeIndex>,
  pub mark_box: Arc<Mutex<MarkBox>>,
  pub module_by_id: HashMap<SmolStr, Box<Module>>,
  pub plugin_driver: Arc<PluginDriver>,
}

// Relation between modules
#[derive(Debug)]
pub enum Rel {
    Import(RelationInfo),
    ReExport(RelationInfo),
    ReExportAll,
}

// impl Rel {
//     #[inline]
//     fn get_order(&self) -> usize {
//         match self {
//             Self::Import(info) => info.order,
//             Self::ReExport(info) => info.order,
//             Self::ReExportAll(order) => *order,
//         }
//     }
// }

pub enum Msg {
    DependencyReference(SmolStr, SmolStr, Rel),
    NewMod(Box<Module>),
    NewExtMod(ExternalModule),
}

impl ModuleGraphContainer {
    pub fn new(bundle_options: Arc<BundleOptions>, plugin_driver: Arc<PluginDriver>, mark_box: Arc<Mutex<MarkBox>>) -> Self {
        Self {
            plugin_driver,
            bundle_options,
            resolved_entries: Default::default(),
            entry_indexs: Default::default(),
            ordered_modules: Default::default(),
            module_by_id: Default::default(),
            relation_graph: ModulePetGraph::new(),
            mark_box,
            path_to_node_idx: Default::default(),
        }
    }

    // build dependency graph via entry modules.
    async fn generate(&mut self) {
        let nums_of_thread = num_cpus::get();
        let idle_thread_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(nums_of_thread));
        let job_queue: Arc<SegQueue<ResolvedId>> = Default::default();
        // self.resolved_entries = self
        //     .bundle_options
        //     .entries
        //     .iter()
        //     .map(|entry| resolve_id(entry, None, false))
        //     .collect();

        self.resolved_entries = join_all(
            self.bundle_options
                .entries
                .iter()
                .map(|entry| resolve_id(entry, None, false, &self.plugin_driver)),
        )
        .await
        .into_iter()
        .collect();

        let path_to_node_idx = &mut self.path_to_node_idx;

        self.resolved_entries.iter().for_each(|resolved_entry_id| {
            let entry_idx = self.relation_graph.add_node(resolved_entry_id.id.clone());
            self.entry_indexs.push(entry_idx);
            path_to_node_idx.insert(resolved_entry_id.id.clone(), entry_idx);
            job_queue.push(resolved_entry_id.clone());
        });

        let processed_id: Arc<DashSet<SmolStr>> = Default::default();

        let (tx, rx) = channel::unbounded::<Msg>();

        for _ in 0..nums_of_thread {
            let idle_thread_count = idle_thread_count.clone();
            let mut worker = Worker {
                tx: tx.clone(),
                job_queue: job_queue.clone(),
                processed_id: processed_id.clone(),
                symbol_box: self.mark_box.clone(),
                plugin_driver: self.plugin_driver.clone(),
            };
            tokio::task::spawn(async move {
                'root: loop {
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
                            .or_insert_with_key(|key| self.relation_graph.add_node(key.clone()));
                        let to_id = *path_to_node_idx
                            .entry(to)
                            .or_insert_with_key(|key| self.relation_graph.add_node(key.clone()));
                        self.relation_graph.add_edge(from_id, to_id, rel);
                    }
                    _ => {}
                }
            }
        }

        let entries_id = self
            .entry_indexs
            .iter()
            .map(|idx| &self.relation_graph[*idx])
            .collect::<HashSet<&SmolStr>>();
        self.module_by_id.par_iter_mut().for_each(|(_key, module)| {
            module.is_user_defined_entry_point = entries_id.contains(&module.id);
        });
    }

    fn sort_modules(&mut self) {
        // FIXME: This is not a complete version
        let mut stack = self
            .resolved_entries
            .iter()
            .map(|rid| rid.id.clone())
            .rev()
            .collect::<Vec<_>>();
        let mut visited = HashSet::new();
        let mut next_exec_order = 0;
        while let Some(id) = stack.pop() {
            let module = self.module_by_id.get_mut(&id).unwrap();
            if !visited.contains(&id) {
                visited.insert(id.clone());
                stack.push(id);
                module
                    .dependencies
                    .keys()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .for_each(|dep| {
                        let rid = module.resolved_ids.get(dep).unwrap().clone();
                        stack.push(rid.id);
                    });
            } else {
                module.exec_order = next_exec_order;
                next_exec_order += 1;
            }
        }
        let mut modules = self.module_by_id.values().collect::<Vec<_>>();
        modules.sort_by_key(|m| m.exec_order);
        log::debug!(
            "ordered {:#?}",
            modules.iter().map(|m| &m.id).collect::<Vec<_>>()
        );
        self.ordered_modules = modules
            .iter()
            .map(|m| *self.path_to_node_idx.get(&m.id).unwrap())
            .collect();
    }

    pub async fn build(mut self) -> ModuleGraph {
        self.generate().await;
        self.sort_modules();
        self.link_module_exports();
        self.link_module();

        ModuleGraph {
          resolved_entries: self.resolved_entries,
          id_to_node_idx: self.path_to_node_idx,
          relation_graph: self.relation_graph,
          // entry_indexs: self.entry_indexs,
          ordered_modules: self.ordered_modules,
          // mark_box: Arc<Mutex<MarkBox>>,
          module_by_id: self.module_by_id,
        }
    }

    pub fn link_module_exports(&mut self) {
        self.ordered_modules.iter().for_each(|idx| {
            let module_id = &self.relation_graph[*idx];
            let module = self.module_by_id.get(module_id).unwrap();
            // self.module_by_id.get_mut
            let dep_ids = module
                .re_export_all_sources
                .iter()
                .map(|dep_src| module.resolved_ids.get(dep_src).unwrap().clone().id)
                .collect::<Vec<_>>();
            let dep_exports = dep_ids
                .into_par_iter()
                .map(|id| self.module_by_id.get(&id).unwrap())
                .map(|dep_module| (dep_module.id.clone(), dep_module.exports.clone()))
                .collect::<Vec<_>>();

            let module = self.module_by_id.get_mut(module_id).unwrap();
            dep_exports.into_iter().for_each(|(dep_id, dep_exports)| {
                dep_exports.into_iter().for_each(|(exported_name, mark)| {
                    assert!(
                        !module.exports.contains_key(&exported_name),
                        "duplicate when export {:?} from {:?} in {:?}",
                        exported_name,
                        dep_id,
                        module.id
                    );
                    module.exports.insert(exported_name, mark);
                });
            });
        });
    }

    pub fn link_module(&mut self) {
        self.ordered_modules.iter().for_each(|idx| {
            let edges = self
                .relation_graph
                .edges_directed(*idx, EdgeDirection::Outgoing);
            edges.for_each(|edge| {
                log::debug!(
                    "[graph]: link module from {:?} to {:?}",
                    &self.relation_graph[*idx],
                    &self.relation_graph[edge.target()]
                );
                let rel_info = match edge.weight() {
                    Rel::Import(info) => Some(info),
                    Rel::ReExport(info) => Some(info),
                    _ => None,
                };
                if let Some(rel_info) = rel_info {
                    rel_info.names.iter().for_each(|specifier| {
                        let dep_module = self
                            .module_by_id
                            .get_mut(&self.relation_graph[edge.target()])
                            .unwrap();
                        // import _default from './foo'
                        // import * as foo from './foo
                        // export * as foo from './foo
                        if &specifier.original == "default" || &specifier.original == "*" {
                            // There is only one case where `specifier.used` is not a valid varible name.
                            // Special case ` export { default } from ...`
                            if &specifier.used != "default" {
                                dep_module.suggest_name(
                                    specifier.original.clone(),
                                    specifier.used.clone(),
                                );
                            }
                        }

                        log::debug!(
                            "[graph]: link imported `{:?}` to exported {} in {}",
                            specifier.used,
                            specifier.original,
                            dep_module.id
                        );

                        if &specifier.original == "*" {
                            // REFACTOR
                            dep_module.include_namespace();
                        }

                        let dep_module_exported_mark = dep_module
                            .exports
                            .get(&specifier.original)
                            .expect("Not found");

                        self.mark_box
                            .lock()
                            .unwrap()
                            .union(specifier.mark, *dep_module_exported_mark);
                    });
                }
            });
        });
    }
}

use std::{
  collections::{HashMap, HashSet},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use crossbeam::{
  channel::{self},
  queue::SegQueue,
};
use dashmap::DashSet;
use futures::future::join_all;
use petgraph::graph::NodeIndex;
use smol_str::SmolStr;
use tracing::instrument;

use crate::{
  bundler::BundleOptions, js_module::JsModule, plugin_driver::PluginDriver, structs::ResolvedId,
  utils::resolve_id, worker2::Worker,
};
pub enum Msg {
  // DependencyReference(SmolStr, SmolStr, Rel),
  NewMod(JsModule),
  // NewExtMod(ExternalModule),
}
#[derive(Debug, Default)]
pub struct ModuleGraph {
  pub resolved_entries: Vec<ResolvedId>,
  pub id_to_node_idx: HashMap<SmolStr, NodeIndex>,
  // pub relation_graph: ModulePetGraph,
  pub ordered_modules: Vec<SmolStr>,
  pub module_by_id: HashMap<SmolStr, JsModule>,
}

impl ModuleGraph {
  pub fn node_idx_of_enties(&self) -> Vec<NodeIndex> {
    self
      .resolved_entries
      .iter()
      .map(|rid| *self.id_to_node_idx.get(&rid.id).unwrap())
      .collect()
  }

  pub fn sort_modules(&mut self) {
    let mut stack = self
      .resolved_entries
      .iter()
      .map(|rid| rid.id.clone())
      .rev()
      .collect::<Vec<_>>();
    let mut dyn_imports = vec![];
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
            let rid = module.resolved_ids().get(dep).unwrap().clone();
            stack.push(rid.id);
          });
        module
          .dyn_dependencies
          .iter()
          .collect::<Vec<_>>()
          .into_iter()
          .rev()
          .for_each(|dep| {
            let rid = module.resolved_ids().get(&dep.argument).unwrap().clone();
            dyn_imports.push(rid.id);
          });
      } else {
        module.exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    stack = dyn_imports.into_iter().rev().collect();
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
            let rid = module.resolved_ids().get(dep).unwrap().clone();
            stack.push(rid.id);
          });
      } else {
        module.exec_order = next_exec_order;
        next_exec_order += 1;
      }
    }
    let mut modules = self.module_by_id.values().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.exec_order);
    tracing::debug!(
      "ordered {:#?}",
      modules.iter().map(|m| &m.id).collect::<Vec<_>>()
    );
    self.ordered_modules = modules.iter().map(|m| m.id.clone()).collect();
  }

  #[instrument]
  pub async fn build_from(
    bundle_options: Arc<BundleOptions>,
    plugin_driver: Arc<PluginDriver>,
  ) -> Self {
    let mut module_graph = Self::default();
    let nums_of_thread = num_cpus::get();
    let idle_thread_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(nums_of_thread));
    let job_queue: Arc<SegQueue<ResolvedId>> = Default::default();

    module_graph.resolved_entries = join_all(
      bundle_options
        .entries
        .iter()
        .map(|entry| resolve_id(entry, None, false, &plugin_driver)),
    )
    .await
    .into_iter()
    .collect();

    let processed_id: Arc<DashSet<SmolStr>> = Default::default();

    module_graph.resolved_entries.iter().for_each(|rd| {
      job_queue.push(rd.clone());
    });

    let (tx, rx) = channel::unbounded::<Msg>();

    for _ in 0..nums_of_thread {
      let idle_thread_count = idle_thread_count.clone();
      let mut worker = Worker {
        tx: tx.clone(),
        job_queue: job_queue.clone(),
        processed_id: processed_id.clone(),
        plugin_driver: plugin_driver.clone(),
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
            module_graph.module_by_id.insert(module.id.clone(), module);
          } // _ => {}
        }
      }
    }

    let entries_id = module_graph
      .resolved_entries
      .iter()
      .map(|rid| rid.id.clone())
      .collect::<HashSet<SmolStr>>();
    module_graph
      .module_by_id
      .iter_mut()
      .for_each(|(_key, module)| {
        module.is_user_defined_entry_point = entries_id.contains(&module.id);
      });

    module_graph.sort_modules();
    module_graph
  }
}

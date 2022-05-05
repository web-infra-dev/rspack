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
use rspack_shared::{JsModule, ModuleGraph};
use smol_str::SmolStr;
use tracing::instrument;

use crate::{
  bundler::BundleOptions, plugin_driver::PluginDriver, structs::ResolvedId, utils::resolve_id,
  worker2::Worker,
};

#[derive(Debug)]
pub enum Msg {
  // DependencyReference(SmolStr, SmolStr, Rel),
  NewMod(JsModule),
  // NewExtMod(ExternalModule),
}

#[derive(Debug, Default)]
pub struct ModuleGraphFactory {
  pub resolved_entries: Vec<ResolvedId>,
  pub id_to_node_idx: HashMap<SmolStr, NodeIndex>,
  // pub relation_graph: ModulePetGraph,
  pub ordered_modules: Vec<SmolStr>,
  pub module_by_id: HashMap<SmolStr, JsModule>,
}

impl ModuleGraphFactory {
  #[instrument(skip(bundle_options, plugin_driver))]
  pub async fn build_from(
    bundle_options: Arc<BundleOptions>,
    plugin_driver: Arc<PluginDriver>,
  ) -> ModuleGraph {
    let mut module_graph = ModuleGraph::default();
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

    let visited_module_id: Arc<DashSet<SmolStr>> = Default::default();

    module_graph.resolved_entries.iter().for_each(|rd| {
      job_queue.push(rd.clone());
    });

    let (tx, rx) = channel::unbounded::<Msg>();

    for _ in 0..nums_of_thread {
      let idle_thread_count = idle_thread_count.clone();
      let mut worker = Worker {
        tx: tx.clone(),
        job_queue: job_queue.clone(),
        visited_module_id: visited_module_id.clone(),
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

  #[instrument(skip(bundle_options, plugin_driver, module_graph, changed_file))]
  pub async fn build_from_cache(
    bundle_options: Arc<BundleOptions>,
    plugin_driver: Arc<PluginDriver>,
    mut module_graph: ModuleGraph,
    changed_file: SmolStr,
  ) -> ModuleGraph {
    let nums_of_thread = num_cpus::get();
    let idle_thread_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(nums_of_thread));
    let job_queue: Arc<SegQueue<ResolvedId>> = Default::default();
    job_queue.push(ResolvedId {
      id: changed_file.clone(),
      external: false,
    });

    let visited_module_id: Arc<DashSet<SmolStr>> = Arc::new(
      module_graph
        .module_by_id
        .keys()
        .cloned()
        .collect::<DashSet<_>>(),
    );
    visited_module_id.remove(&changed_file);

    let (tx, rx) = channel::unbounded::<Msg>();

    for _ in 0..nums_of_thread {
      let idle_thread_count = idle_thread_count.clone();
      let mut worker = Worker {
        tx: tx.clone(),
        job_queue: job_queue.clone(),
        visited_module_id: visited_module_id.clone(),
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

    // TODO: We need to remove nodes without edges.

    module_graph.sort_modules();
    module_graph
  }
}

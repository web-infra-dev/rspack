use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};

use crate::task::Task;
use crate::{
  plugin_hook, BundleContext, BundleOptions, JsModule, ModuleGraph, PluginDriver, ResolvedId,
};
use crossbeam::channel::{self};
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use smol_str::SmolStr;

#[derive(Debug)]
pub struct Bundle {
  entries: Vec<String>,
  pub options: Arc<BundleOptions>,
  pub context: Arc<BundleContext>,
  pub plugin_driver: Arc<PluginDriver>,
  pub module_graph: Option<ModuleGraph>,
}

#[derive(Debug)]
pub enum Msg {
  // DependencyReference(SmolStr, SmolStr, Rel),
  TaskFinished(JsModule),
  // NewExtMod(ExternalModule),
}

impl Bundle {
  pub fn new(
    options: Arc<BundleOptions>,
    plugin_driver: Arc<PluginDriver>,
    context: Arc<BundleContext>,
  ) -> Self {
    Self {
      entries: options.entries.clone(),
      plugin_driver,
      context,
      options,
      module_graph: Default::default(),
    }
  }

  pub fn add_entry(&mut self, entry: String) {
    self.entries.push(entry);
  }

  pub async fn build_graph(&mut self) {
    let mut module_graph = ModuleGraph::default();
    let active_task_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let job_queue: Arc<SegQueue<ResolvedId>> = Default::default();

    module_graph.resolved_entries = join_all(
      self
        .entries
        .iter()
        .map(|entry| plugin_hook::resolve_id(entry, None, false, &self.plugin_driver)),
    )
    .await
    .into_iter()
    .collect();

    let visited_module_id: Arc<DashSet<SmolStr>> = Default::default();

    module_graph.resolved_entries.iter().for_each(|rd| {
      job_queue.push(rd.clone());
    });

    let (tx, rx) = channel::unbounded::<Msg>();

    while let Some(job) = job_queue.pop() {
      visited_module_id.insert(job.id.clone());
      active_task_count.fetch_add(1, Ordering::SeqCst);
      let mut task = Task {
        resolved_id: job,
        active_task_count: active_task_count.clone(),
        visited_module_id: visited_module_id.clone(),
        tx: tx.clone(),
        plugin_driver: self.plugin_driver.clone(),
      };

      tokio::task::spawn(async move {
        task.run().await;
      });
    }

    while active_task_count.load(Ordering::SeqCst) != 0 || !rx.is_empty() {
      if let Ok(job) = rx.recv() {
        match job {
          Msg::TaskFinished(module) => {
            module_graph.module_by_id.insert(module.id.clone(), module);
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          _ => {}
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
    self.module_graph = Some(module_graph);
  }
}

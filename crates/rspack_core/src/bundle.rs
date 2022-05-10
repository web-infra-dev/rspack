use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};

use crate::task::Task;
use crate::{
  plugin_hook, BundleContext, JsModule, ModuleGraph, NormalizedBundleOptions, PluginDriver,
  ResolvedId,
};
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use tracing::instrument;

#[derive(Debug)]
pub struct Bundle {
  entries: Vec<String>,
  pub options: Arc<NormalizedBundleOptions>,
  pub context: Arc<BundleContext>,
  pub plugin_driver: Arc<PluginDriver>,
  pub module_graph: Option<ModuleGraph>,
}

#[derive(Debug)]
pub enum Msg {
  // DependencyReference(String, String, Rel),
  TaskFinished(JsModule),
  // NewExtMod(ExternalModule),
}

impl Bundle {
  pub fn new(
    options: Arc<NormalizedBundleOptions>,
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

  #[instrument(skip(self))]
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

    let visited_module_id: Arc<DashSet<String>> = Default::default();

    module_graph.resolved_entries.iter().for_each(|rd| {
      job_queue.push(rd.clone());
    });

    // let (tx, rx) = channel::unbounded::<Msg>();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    while let Some(job) = job_queue.pop() {
      visited_module_id.insert(job.path.clone());
      active_task_count.fetch_add(1, Ordering::SeqCst);
      let mut task = Task {
        root: self.options.root.clone(),
        resolved_id: job,
        active_task_count: active_task_count.clone(),
        visited_module_id: visited_module_id.clone(),
        tx: tx.clone(),
        plugin_driver: self.plugin_driver.clone(),
        code_splitting: self.options.code_splitting,
      };
      tokio::task::spawn(async move {
        task.run().await;
      });
    }

    let entries_id = module_graph
      .resolved_entries
      .iter()
      .map(|rid| rid.path.clone())
      .collect::<HashSet<String>>();

    while active_task_count.load(Ordering::SeqCst) != 0 {
      match rx.recv().await {
        Some(job) => match job {
          Msg::TaskFinished(mut module) => {
            module.is_user_defined_entry_point = entries_id.contains(&module.path);
            if module.is_user_defined_entry_point {
              tracing::trace!("detect user entry module {:?}", module);
            }
            module_graph
              .module_by_id
              .insert(module.path.clone(), module);
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          _ => {}
        },
        None => {
          tracing::trace!("All sender is dropped");
        }
      }
    }

    module_graph.sort_modules();
    self.module_graph = Some(module_graph);
  }
}

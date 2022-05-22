use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};

use crate::task::Task;
use crate::{
  plugin_hook, BundleContext, ImportKind, JsModule, ModuleGraph, NormalizedBundleOptions,
  PluginDriver, ResolvedURI,
};
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use nodejs_resolver::Resolver;
use tracing::instrument;

#[derive(Debug)]
pub struct Bundle {
  entries: Vec<String>,
  pub options: Arc<NormalizedBundleOptions>,
  pub context: Arc<BundleContext>,
  pub plugin_driver: Arc<PluginDriver>,
  pub module_graph: ModuleGraph,
  pub visited_module_id: Arc<DashSet<String>>,
  pub resolver: Arc<Resolver>,
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
    resolver: Arc<Resolver>,
  ) -> Self {
    Self {
      entries: options.entries.clone(),
      plugin_driver,
      context,
      options,
      module_graph: Default::default(),
      visited_module_id: Default::default(),
      resolver,
    }
  }

  pub fn add_entry(&mut self, entry: String) {
    self.entries.push(entry);
  }

  #[instrument(skip(self))]
  pub async fn build_graph(&mut self, changed_files: Option<Vec<String>>) {
    let active_task_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let job_queue: Arc<SegQueue<ResolvedURI>> = Default::default();

    if let Some(files) = changed_files {
      files.into_iter().for_each(|rd| {
        job_queue.push(ResolvedURI {
          uri: rd,
          kind: ImportKind::Import,
          external: false,
        });
      });
    }

    self.module_graph.resolved_entries = join_all(self.entries.iter().map(|entry| {
      plugin_hook::resolve_id(
        crate::ResolveArgs {
          id: entry.clone(),
          importer: None,
          kind: ImportKind::Import,
        },
        false,
        &self.plugin_driver,
        &self.resolver,
      )
    }))
    .await
    .into_iter()
    .collect();

    self.module_graph.resolved_entries.iter().for_each(|rd| {
      job_queue.push(rd.clone());
    });

    // let (tx, rx) = channel::unbounded::<Msg>();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    while let Some(job) = job_queue.pop() {
      self.visited_module_id.insert(job.uri.clone());
      active_task_count.fetch_add(1, Ordering::SeqCst);
      let mut task = Task {
        root: self.options.root.clone(),
        resolved_uri: job,
        active_task_count: active_task_count.clone(),
        visited_module_uri: self.visited_module_id.clone(),
        tx: tx.clone(),
        plugin_driver: self.plugin_driver.clone(),
        resolver: self.resolver.clone(),
      };
      tokio::task::spawn(async move {
        task.run().await;
      });
    }

    let entries_uri = self
      .module_graph
      .resolved_entries
      .iter()
      .map(|rid| rid.uri.clone())
      .collect::<HashSet<String>>();

    while active_task_count.load(Ordering::SeqCst) != 0 {
      match rx.recv().await {
        Some(job) => match job {
          Msg::TaskFinished(mut module) => {
            module.is_user_defined_entry_point = entries_uri.contains(&module.uri);
            if module.is_user_defined_entry_point {
              tracing::trace!("detect user entry module {:?}", module);
            }
            self
              .module_graph
              .module_by_id
              .insert(module.uri.clone(), module);
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
        },
        None => {
          tracing::trace!("All sender is dropped");
        }
      }
    }

    self.module_graph.sort_modules();
  }
}

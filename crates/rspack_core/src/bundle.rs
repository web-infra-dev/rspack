use crate::path::gen_module_id;
use crate::task::Task;
use crate::{
  plugin_hook, BundleContext, BundleEntries, ChunkGraph, EntryItem, ImportKind, JsModule,
  JsModuleKind, ModuleGraphContainer, ModuleIdAlgo, NormalizedBundleOptions, PluginDriver,
  ResolveArgs, ResolvedURI,
};
use crossbeam::queue::SegQueue;
use dashmap::DashSet;
use futures::future::join_all;
use nodejs_resolver::Resolver;
use rspack_swc::swc_atoms::JsWord;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};
use sugar_path::PathSugar;
use tracing::instrument;
#[derive(Debug)]
pub struct Bundle {
  entries: BundleEntries,
  pub options: Arc<NormalizedBundleOptions>,
  pub context: Arc<BundleContext>,
  pub plugin_driver: Arc<PluginDriver>,
  pub module_graph_container: ModuleGraphContainer,
  pub chunk_graph: ChunkGraph,
  pub visited_module_id: Arc<DashSet<String>>,
  pub resolver: Arc<Resolver>,
}

#[derive(Debug)]
pub enum Msg {
  DependencyReference((ResolveArgs, ResolvedURI)),
  TaskFinished(JsModule),
  CanCel(), // NewExtMod(ExternalModule),
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
      chunk_graph: Default::default(),
      module_graph_container: Default::default(),
      visited_module_id: Default::default(),
      resolver,
    }
  }

  pub fn add_entry(&mut self, entry: (String, EntryItem)) {
    self.entries.insert(entry.0, entry.1);
  }

  #[instrument(skip(self))]
  pub async fn build_graph(&mut self, changed_files: Option<Vec<String>>) {
    let active_task_count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let job_queue: Arc<SegQueue<ResolveArgs>> = Default::default();
    let mut resolved_map: HashMap<ResolveArgs, ResolvedURI> = Default::default();

    if let Some(files) = changed_files {
      files.into_iter().for_each(|rd| {
        job_queue.push(ResolveArgs {
          id: rd,
          kind: ImportKind::Import,
          importer: None,
        });
      });
    }

    self.module_graph_container.resolved_entries = self
      .entries
      .iter()
      .map(|(name, entry)| {
        (
          name.clone(),
          ResolvedURI {
            uri: Path::new(&self.options.root)
              .join(entry.src.clone())
              .normalize()
              .to_string_lossy()
              .to_string(),
            kind: ImportKind::Import,
            external: false,
          },
        )
      })
      .collect();

    self.entries.iter().for_each(|(name, entry)| {
      job_queue.push(ResolveArgs {
        id: entry.src.clone(),
        kind: ImportKind::Import,
        importer: None,
      });
    });

    // let (tx, rx) = channel::unbounded::<Msg>();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    while let Some(job) = job_queue.pop() {
      active_task_count.fetch_add(1, Ordering::SeqCst);
      let mut task = Task {
        root: self.options.root.clone(),
        resolve_args: job,
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
      .module_graph_container
      .resolved_entries
      .iter()
      .map(|(name, rid)| (rid.uri.clone(), name.clone()))
      .collect::<HashMap<_, _>>();

    let mut module_id_count = 0;
    while active_task_count.load(Ordering::SeqCst) != 0 {
      match rx.recv().await {
        Some(job) => match job {
          Msg::CanCel() => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          Msg::TaskFinished(mut module) => {
            module.kind = entries_uri
              .get(&module.uri)
              .map_or(JsModuleKind::Normal, |name| JsModuleKind::UserEntry {
                name: name.clone(),
              });
            module.id = match self.context.options.optimization.module_id_algo {
              ModuleIdAlgo::Numeric => {
                module_id_count += 1;
                module_id_count.to_string()
              }
              ModuleIdAlgo::Named => gen_module_id(&self.context.options.root, &module.uri),
            };
            self.module_graph_container.module_graph.add_module(module);
            active_task_count.fetch_sub(1, Ordering::SeqCst);
          }
          Msg::DependencyReference((resolve_args, resolved_uri)) => {
            resolved_map.insert(resolve_args, resolved_uri);
          }
        },
        None => {
          tracing::trace!("All sender is dropped");
        }
      }
    }

    resolved_map
      .into_iter()
      .for_each(|(resolve_args, resolved_uri)| {
        if let Some(importer) = resolve_args.importer {
          if let Some(module) = self
            .module_graph_container
            .module_graph
            .module_by_uri_mut(&importer)
          {
            module
              .resolved_uris
              .insert(JsWord::from(resolve_args.id), resolved_uri);
          }
        }
      });

    self.module_graph_container.sort_modules();
  }
}

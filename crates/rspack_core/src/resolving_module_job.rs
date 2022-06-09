use std::{
  path::Path,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use dashmap::DashSet;
use sugar_path::PathSugar;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  load, resolve, LoadArgs, ModuleGraphModule, Msg, ParseModuleArgs, PluginDriver, ResolveArgs,
  SourceType,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Dependency {
  /// Uri of importer module
  pub importer: Option<String>,
  /// `./a.js` in `import './a.js'` is specifier
  pub specifier: String,
  pub kind: ResolveKind,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ResolveKind {
  Import,
  Require,
  DynamicImport,
  AtImport,
}

pub struct ResolvingModuleJob {
  context: JobContext,
  dependency: Dependency,
  tx: UnboundedSender<Msg>,
  plugin_driver: Arc<PluginDriver>,
}

impl ResolvingModuleJob {
  pub fn new(
    context: JobContext,
    dependency: Dependency,
    tx: UnboundedSender<Msg>,
    plugin_driver: Arc<PluginDriver>,
  ) -> Self {
    context.active_task_count.fetch_add(1, Ordering::SeqCst);

    Self {
      context,
      dependency,
      tx,
      plugin_driver,
    }
  }
  pub async fn run(mut self) {
    // reoslve to get id

    // TODO: caching in resolve
    let uri = resolve(
      ResolveArgs {
        importer: self.dependency.importer.as_deref(),
        specifier: self.dependency.specifier.as_str(),
        kind: self.dependency.kind,
      },
      &self.plugin_driver,
    );
    tracing::trace!("resolved uri {:?}", uri);

    self.context.source_type = resolve_source_type_by_uri(uri.as_str());

    self
      .tx
      .send(Msg::DependencyReference(
        self.dependency.clone(),
        uri.clone(),
      ))
      .unwrap();

    // load source by id

    if self.context.visited_module_uri.contains(&uri) {
      self.tx.send(Msg::TaskErrorEncountered(())).unwrap();
    } else {
      self.context.visited_module_uri.insert(uri.clone());
      let source = load(LoadArgs { uri: uri.as_str() }).await;
      tracing::trace!(
        "load ({:?}) source {:?}",
        self.context.source_type,
        &source[0..usize::min(source.len(), 20)]
      );

      // TODO: transform

      // parse source to module

      assert!(self.context.source_type.is_some());

      let mut module = self
        .plugin_driver
        .parse_module(
          ParseModuleArgs {
            uri: uri.as_str(),
            source,
          },
          &mut self.context,
        )
        .unwrap();

      tracing::trace!("parsed module {:?}", module);

      // scan deps

      let deps = module
        .dependencies()
        .into_iter()
        .map(|dep| Dependency {
          importer: Some(uri.clone()),
          specifier: dep.specifier,
          kind: dep.kind,
        })
        .collect::<Vec<_>>();

      tracing::trace!("get deps {:?}", deps);
      deps.iter().for_each(|dep| {
        self.fork(dep.clone());
      });

      self
        .tx
        .send(Msg::TaskFinished(Box::new(ModuleGraphModule::new(
          Path::new("./")
            .join(Path::new(uri.as_str()).relative(self.plugin_driver.options.root.as_str()))
            .to_string_lossy()
            .to_string(),
          uri,
          module,
          deps,
          self.context.source_type.unwrap(),
        ))))
        .unwrap();

      // make module
      // Ok(())
    }
  }

  fn fork(&self, dep: Dependency) {
    let context = self.context.clone();
    let task = ResolvingModuleJob::new(context, dep, self.tx.clone(), self.plugin_driver.clone());

    tokio::task::spawn(async move {
      task.run().await;
    });
  }
}

pub fn resolve_source_type_by_uri<T: AsRef<Path>>(uri: T) -> Option<SourceType> {
  let uri = uri.as_ref();
  let ext = uri.extension()?.to_str()?;
  let source_type: Option<SourceType> = ext.try_into().ok();
  source_type
}

#[derive(Debug, Clone)]
pub struct JobContext {
  pub importer: Option<String>,
  pub(crate) active_task_count: Arc<AtomicUsize>,
  pub(crate) visited_module_uri: Arc<DashSet<String>>,
  pub source_type: Option<SourceType>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ModuleDependency {
  pub specifier: String,
  pub kind: ResolveKind,
}

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
  load, parse_to_url, resolve, LoadArgs, ModuleGraphModule, Msg, ParseModuleArgs, PluginDriver,
  ResolveArgs, SourceType, VisitedModuleIdentity,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Dependency {
  /// Uri of importer module
  pub importer: Option<String>,
  pub detail: ModuleDependency,
}

// impl Dependency {
//   pub fn new(importer: Option<String>, specifier: String, kind: ResolveKind) -> Self {
//     Self {
//       importer,
//       specifier,
//       kind,
//     }
//   }
// }

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ResolveKind {
  Import,
  Require,
  DynamicImport,
  AtImport,
  AtImportUrl,
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
    match self.resolve_module().await {
      Ok(maybe_module) => {
        if let Some(module) = maybe_module {
          self.send(Msg::TaskFinished(Box::new(module)));
        } else {
          self.send(Msg::TaskCanceled);
        }
      }
      Err(err) => self.send(Msg::TaskErrorEncountered(err)),
    }
  }

  pub fn send(&self, msg: Msg) {
    if let Err(err) = self.tx.send(msg) {
      tracing::trace!("fail to send msg {:?}", err)
    }
  }

  pub async fn resolve_module(&mut self) -> anyhow::Result<Option<ModuleGraphModule>> {
    // TODO: caching in resolve

    let uri = resolve(
      ResolveArgs {
        importer: self.dependency.importer.as_deref(),
        specifier: self.dependency.detail.specifier.as_str(),
        kind: self.dependency.detail.kind,
      },
      &self.plugin_driver,
      &mut self.context,
    )
    .await?;
    tracing::trace!("resolved uri {:?}", uri);

    if self.context.source_type.is_none() {
      let url = parse_to_url(&uri);
      assert_eq!(url.scheme(), "specifier");
      self.context.source_type = resolve_source_type_by_uri(url.path());
    }

    self
      .tx
      .send(Msg::DependencyReference(
        self.dependency.clone(),
        uri.clone(),
      ))
      .unwrap();

    if self
      .context
      .visited_module_identity
      .contains(&(uri.clone(), self.dependency.detail.clone()))
    {
      return Ok(None);
    }

    self
      .context
      .visited_module_identity
      .insert((uri.clone(), self.dependency.detail.clone()));

    let source = load(
      &self.plugin_driver,
      LoadArgs { uri: uri.as_str() },
      &mut self.context,
    )
    .await?;
    tracing::trace!(
      "load ({:?}) source {:?}",
      self.context.source_type,
      &source[0..usize::min(source.len(), 20)]
    );

    // TODO: transform

    let mut module = self.plugin_driver.parse_module(
      ParseModuleArgs {
        uri: uri.as_str(),
        source,
      },
      &mut self.context,
    )?;

    tracing::trace!("parsed module {:?}", module);

    // scan deps

    let deps = module
      .dependencies()
      .into_iter()
      .map(|dep| Dependency {
        importer: Some(uri.clone()),
        detail: dep,
      })
      .collect::<Vec<_>>();

    tracing::trace!("get deps {:?}", deps);
    deps.iter().for_each(|dep| {
      self.fork(dep.clone());
    });

    let resolved_module = ModuleGraphModule::new(
      self.context.module_name.clone(),
      Path::new("./")
        .join(Path::new(uri.as_str()).relative(self.plugin_driver.options.root.as_str()))
        .to_string_lossy()
        .to_string(),
      uri,
      module,
      deps,
      self
        .context
        .source_type
        .ok_or_else(|| anyhow::format_err!("source type is empty"))?,
    );
    Ok(Some(resolved_module))
  }

  fn fork(&self, dep: Dependency) {
    let task = ResolvingModuleJob::new(
      JobContext {
        module_name: None,
        source_type: None,
        ..self.context.clone()
      },
      dep,
      self.tx.clone(),
      self.plugin_driver.clone(),
    );

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
  pub module_name: Option<String>,
  pub(crate) active_task_count: Arc<AtomicUsize>,
  pub(crate) visited_module_identity: VisitedModuleIdentity,
  pub source_type: Option<SourceType>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ModuleDependency {
  pub specifier: String,
  /// `./a.js` in `import './a.js'` is specifier
  pub kind: ResolveKind,
}

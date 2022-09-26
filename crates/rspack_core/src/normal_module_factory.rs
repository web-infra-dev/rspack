use std::{
  path::{Path, PathBuf},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use crate::{
  BoxModule, CompilerOptions, FactorizeAndBuildArgs, LoaderResult, LoaderRunnerRunner, ResourceData,
};
use rspack_error::{Diagnostic, Error, TWithDiagnosticArray};
use rspack_sources::{
  BoxSource, OriginalSource, RawSource, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};
use sugar_path::PathSugar;
use swc_common::Span;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  // load,
  parse_to_url,
  resolve,
  Content,
  ModuleGraphModule,
  ModuleType,
  Msg,
  ParseModuleArgs,
  PluginDriver,
  ResolveArgs,
  VisitedModuleIdentity,
};
use rspack_error::Result;

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
  UrlToken,
}

pub struct NormalModuleFactory {
  context: NormalModuleFactoryContext,
  dependency: Dependency,
  tx: UnboundedSender<Msg>,
  plugin_driver: Arc<PluginDriver>,
  loader_runner_runner: Arc<LoaderRunnerRunner>,
  diagnostic: Vec<Diagnostic>,
}

impl NormalModuleFactory {
  pub fn new(
    context: NormalModuleFactoryContext,
    dependency: Dependency,
    tx: UnboundedSender<Msg>,
    plugin_driver: Arc<PluginDriver>,
    loader_runner_runner: Arc<LoaderRunnerRunner>,
  ) -> Self {
    context.active_task_count.fetch_add(1, Ordering::SeqCst);

    Self {
      context,
      dependency,
      tx,
      plugin_driver,
      loader_runner_runner,
      diagnostic: vec![],
    }
  }

  pub async fn run(mut self) {
    match self.resolve_module().await {
      Ok(maybe_module) => {
        if let Some(module) = maybe_module {
          let diagnostic = std::mem::take(&mut self.diagnostic);
          self.send(Msg::TaskFinished(TWithDiagnosticArray::new(
            Box::new(module),
            diagnostic,
          )));
        } else {
          self.send(Msg::TaskCanceled);
        }
      }
      Err(err) => self.send(Msg::TaskErrorEncountered(err)),
    }
  }

  pub fn add_diagnostic<T: Into<Diagnostic>>(&mut self, diagnostic: T) {
    self.diagnostic.push(diagnostic.into());
  }

  pub fn send(&self, msg: Msg) {
    if let Err(err) = self.tx.send(msg) {
      tracing::trace!("fail to send msg {:?}", err)
    }
  }

  pub fn calculate_module_type(&self, uri: &str) -> Option<ModuleType> {
    // todo currently unreachable module types are temporarily unified with their importers
    let url = parse_to_url(if uri.starts_with("UnReachable:") {
      match self.dependency.importer.as_deref() {
        Some(u) => u,
        None => uri,
      }
    } else {
      uri
    });
    debug_assert_eq!(url.scheme().map(|item| item.as_str()), Some("specifier"));
    resolve_module_type_by_uri(PathBuf::from(url.path().as_str()))
  }

  pub async fn factorize_normal_module(&mut self) -> Result<Option<(String, BoxModule)>> {
    let uri = resolve(
      ResolveArgs {
        importer: self.dependency.importer.as_deref(),
        specifier: self.dependency.detail.specifier.as_str(),
        kind: self.dependency.detail.kind,
        span: self.dependency.detail.span,
      },
      &self.plugin_driver,
      &mut self.context,
    )
    .await?;
    tracing::trace!("resolved uri {:?}", uri);

    let url = parse_to_url(&uri);
    if self.context.module_type.is_none() {
      self.context.module_type = self.calculate_module_type(&uri);
    }

    self
      .tx
      .send(Msg::DependencyReference(
        self.dependency.clone(),
        uri.clone(),
      ))
      .map_err(|_| {
        Error::InternalError(format!(
          "Failed to resolve dependency {:?}",
          self.dependency
        ))
      })?;

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

    let resource_data = ResourceData {
      resource: uri.clone(),
      resource_path: url.path().to_string(),
      resource_query: url.query().map(|q| q.to_string()),
      resource_fragment: url.fragment().map(|f| f.to_string()),
    };

    let runner_result = if uri.starts_with("UnReachable:") {
      LoaderResult {
        content: Content::Buffer("module.exports = {}".to_string().as_bytes().to_vec()),
        source_map: None,
        meta: None,
      }
    } else {
      let (runner_result, resolved_module_type) =
        self.loader_runner_runner.run(resource_data).await?;

      self.context.module_type = resolved_module_type;

      if self.context.module_type.is_none() {
        // todo currently unreachable module types are temporarily unified with their importers
        let url = parse_to_url(if uri.starts_with("UnReachable:") {
          self.dependency.importer.as_deref().unwrap()
        } else {
          &uri
        });
        debug_assert_eq!(url.scheme().unwrap().as_str(), "specifier");
        // TODO: remove default module type resolution based on the file extension.
        self.context.module_type = resolve_module_type_by_uri(PathBuf::from(url.path().as_str()));
      }

      runner_result
    };

    tracing::trace!(
      "load ({:?}) source {:?}",
      self.context.module_type,
      runner_result
    );
    let source = self.create_source(&uri, runner_result.content, runner_result.source_map)?;
    let module = self.plugin_driver.parse(
      ParseModuleArgs {
        uri: uri.as_str(),
        meta: runner_result.meta,
        options: self.context.options.clone(),
        source,
        // ast: transform_result.ast.map(|x| x.into()),
      },
      &mut self.context,
    )?;
    tracing::trace!("parsed module {:?}", module);

    Ok(Some((uri, module)))
  }

  fn create_source(
    &self,
    uri: &str,
    content: Content,
    source_map: Option<SourceMap>,
  ) -> Result<BoxSource> {
    match (self.context.options.devtool, content, source_map) {
      (true, content, Some(map)) => {
        let content = content.try_into_string()?;
        Ok(
          SourceMapSource::new(WithoutOriginalOptions {
            value: content,
            name: uri,
            source_map: map,
          })
          .boxed(),
        )
      }
      (true, Content::String(content), None) => Ok(OriginalSource::new(content, uri).boxed()),
      (true, Content::Buffer(content), None) => Ok(RawSource::from(content).boxed()),
      (_, Content::String(content), _) => Ok(RawSource::from(content).boxed()),
      (_, Content::Buffer(content), _) => Ok(RawSource::from(content).boxed()),
    }
  }

  pub async fn resolve_module(&mut self) -> Result<Option<ModuleGraphModule>> {
    // TODO: caching in resolve
    // Here is the corresponding create function in webpack, but instead of using hooks we use procedural functions
    let (uri, mut module) = if let Ok(Some(module)) = self.plugin_driver.factorize_and_build(
      FactorizeAndBuildArgs {
        dependency: &self.dependency,
        plugin_driver: &self.plugin_driver,
      },
      &mut self.context,
    ) {
      let (uri, module) = module;
      self
        .tx
        .send(Msg::DependencyReference(
          self.dependency.clone(),
          uri.clone(),
        ))
        .map_err(|_| {
          Error::InternalError(format!(
            "Failed to resolve dependency {:?}",
            self.dependency
          ))
        })?;
      (uri, module)
    } else if let Some(re) = self.factorize_normal_module().await? {
      re
    } else {
      return Ok(None);
    };

    // let source = load(
    //   &self.plugin_driver,
    //   LoadArgs { uri: uri.as_str() },
    //   &mut self.context,
    // )
    // .await?;
    // tracing::trace!("load ({:?}) source {:?}", self.context.module_type, source);

    // TODO: transform
    // let transform_result = self.plugin_driver.transform(
    //   TransformArgs {
    //     uri: &uri,
    //     content: Some(source),
    //     ast: None,
    //   },
    //   &mut self.context,
    // )?;

    // let mut module = self.plugin_driver.parse(
    //   ParseModuleArgs {
    //     uri: uri.as_str(),
    //     // source: transform_result.content,
    //     options: self.context.options.clone(),
    //     source: runner_result.content,
    //     // ast: transform_result.ast.map(|x| x.into()),
    //   },
    //   &mut self.context,
    // )?;

    // tracing::trace!("parsed module {:?}", module);

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
        .join(Path::new(uri.as_str()).relative(self.plugin_driver.options.context.as_str()))
        .to_string_lossy()
        .to_string(),
      uri,
      module,
      deps,
      self
        .context
        .module_type
        .ok_or_else(|| Error::InternalError("source type is empty".to_string()))?,
    );

    Ok(Some(resolved_module))
  }

  fn fork(&self, dep: Dependency) {
    let task = NormalModuleFactory::new(
      NormalModuleFactoryContext {
        module_name: None,
        module_type: None,
        ..self.context.clone()
      },
      dep,
      self.tx.clone(),
      self.plugin_driver.clone(),
      self.loader_runner_runner.clone(),
    );

    tokio::task::spawn(async move {
      task.run().await;
    });
  }
}

pub fn resolve_module_type_by_uri<T: AsRef<Path>>(uri: T) -> Option<ModuleType> {
  let uri = uri.as_ref();
  let ext = uri.extension()?.to_str()?;
  let module_type: Option<ModuleType> = ext.try_into().ok();
  module_type
}

#[derive(Debug, Clone)]
pub struct NormalModuleFactoryContext {
  pub module_name: Option<String>,
  pub(crate) active_task_count: Arc<AtomicUsize>,
  pub(crate) visited_module_identity: VisitedModuleIdentity,
  pub module_type: Option<ModuleType>,
  pub side_effects: Option<bool>,
  pub options: Arc<CompilerOptions>,
}

#[derive(Debug, Clone, Eq)]
pub struct ModuleDependency {
  pub specifier: String,
  /// `./a.js` in `import './a.js'` is specifier
  pub kind: ResolveKind,
  pub span: Option<ErrorSpan>,
}

/// # WARNING
/// Don't update the manual implementation of `Hash` of [ModuleDependency]
/// Current implementation strong rely on the field of `specifier` and `kind`
impl std::hash::Hash for ModuleDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.specifier.hash(state);
    self.kind.hash(state);
  }
}
/// # WARNING
/// Don't update the manual implementation of `PartialEq` of [ModuleDependency]
/// Current implementation strong rely on the field of `specifier` and `kind`
impl PartialEq for ModuleDependency {
  fn eq(&self, other: &Self) -> bool {
    self.specifier == other.specifier && self.kind == other.kind
  }
}

/// Using `u32` instead of `usize` to reduce memory usage,
/// `u32` is 4 bytes on 64bit machine, comare to `usize` which is 8 bytes.
/// Rspan aka `Rspack span`, just avoiding conflict with span in other crate
/// ## Warning
/// RSpan is zero based, `Span` of `swc` is 1 based. see https://swc-css.netlify.app/?code=eJzLzC3ILypRSFRIK8rPVVAvSS0u0csqVgcAZaoIKg
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub struct ErrorSpan {
  pub start: u32,
  pub end: u32,
}

impl ErrorSpan {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }
}

impl From<Span> for ErrorSpan {
  fn from(span: Span) -> Self {
    Self {
      start: (span.lo.0 as u32).saturating_sub(1),
      end: (span.hi.0 as u32).saturating_sub(1),
    }
  }
}

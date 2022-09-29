use std::{
  path::{Path, PathBuf},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
  },
};

use sugar_path::PathSugar;
use swc_common::Span;
use tokio::sync::mpsc::UnboundedSender;

use rspack_error::{Diagnostic, Error, Result, TWithDiagnosticArray};
use rspack_loader_runner::Loader;

use crate::{
  parse_to_url, resolve, BuildContext, CompilationContext, CompilerContext, CompilerOptions,
  FactorizeAndBuildArgs, LoaderRunnerRunner, ModuleGraphModule, ModuleRule, ModuleType, Msg,
  NormalModule, ResolveArgs, ResourceData, SharedPluginDriver, VisitedModuleIdentity,
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
  UrlToken,
}

#[derive(Debug)]
pub struct NormalModuleFactory {
  context: NormalModuleFactoryContext,
  dependency: Dependency,
  plugin_driver: SharedPluginDriver,
  loader_runner_runner: Arc<LoaderRunnerRunner>,
  diagnostic: Vec<Diagnostic>,
}

impl NormalModuleFactory {
  pub fn new(
    context: NormalModuleFactoryContext,
    dependency: Dependency,
    plugin_driver: SharedPluginDriver,
    loader_runner_runner: Arc<LoaderRunnerRunner>,
  ) -> Self {
    Self {
      context,
      dependency,
      plugin_driver,
      loader_runner_runner,
      diagnostic: vec![],
    }
  }

  pub async fn run(mut self, tx: UnboundedSender<Msg>) {
    match self.resolve_module(&tx).await {
      Ok(maybe_module) => {
        if let Some((next_tasks, module)) = maybe_module {
          let diagnostic = std::mem::take(&mut self.diagnostic);
          self.send(
            &tx,
            Msg::TaskFinished((
              next_tasks,
              TWithDiagnosticArray::new(Box::new(module), diagnostic),
            )),
          );
        } else {
          self.send(&tx, Msg::TaskCanceled);
        }
      }
      Err(err) => self.send(&tx, Msg::TaskErrorEncountered(err)),
    }
  }

  pub fn add_diagnostic<T: Into<Diagnostic>>(&mut self, diagnostic: T) {
    self.diagnostic.push(diagnostic.into());
  }

  pub fn add_diagnostics<T: IntoIterator<Item = Diagnostic>>(&mut self, diagnostic: T) {
    self.diagnostic.extend(diagnostic);
  }

  pub fn send(&self, tx: &UnboundedSender<Msg>, msg: Msg) {
    if let Err(err) = tx.send(msg) {
      tracing::trace!("fail to send msg {:?}", err)
    }
  }

  pub fn calculate_module_type_by_uri(&self, uri: &str) -> Option<ModuleType> {
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

  pub async fn factorize_normal_module(
    &mut self,
    tx: &UnboundedSender<Msg>,
  ) -> Result<Option<(String, NormalModule)>> {
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
      self.context.module_type = self.calculate_module_type_by_uri(&uri);
    }

    tx.send(Msg::DependencyReference(
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

    let resolved_module_type =
      self.calculate_module_type(&resource_data, self.context.module_type)?;

    let resolved_parser_and_generator = self
      .plugin_driver
      .read()
      .await
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        Error::InternalError(format!(
          "Parser and generator builder for module type {:?} is not registered",
          resolved_module_type
        ))
      })?();

    self.context.module_type = Some(resolved_module_type);

    let normal_module = NormalModule::new(
      uri.clone(),
      uri.clone(),
      self.dependency.detail.specifier.to_owned(),
      resolved_module_type,
      resolved_parser_and_generator,
      resource_data,
      self.context.options.clone(),
    );

    Ok(Some((uri, normal_module)))
  }

  pub fn calculate_loaders(
    &self,
    resource_data: &ResourceData,
  ) -> Result<Vec<&dyn Loader<CompilerContext, CompilationContext>>> {
    let resolved_loaders = self
      .context
      .options
      .module
      .rules
      .iter()
      .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
        if let Some(func) = &module_rule.func__ {
          match func(resource_data) {
            Ok(result) => {
              if result {
                return Some(Ok(module_rule));
              }

              return None
            },
            Err(e) => {
              return Some(Err(e.into()))
            }
          }
        }

        // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
        // See: https://webpack.js.org/configuration/module/#ruletest
        if let Some(test_rule) = &module_rule.test && test_rule.is_match(&resource_data.resource) {
          return Some(Ok(module_rule));
        } else if let Some(resource_rule) = &module_rule.resource && resource_rule.is_match(&resource_data.resource) {
          return Some(Ok(module_rule));
        }

        if let Some(resource_query_rule) = &module_rule.resource_query && let Some(resource_query) = &resource_data.resource_query && resource_query_rule.is_match(resource_query) {
          return Some(Ok(module_rule));
        }


        None
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .flat_map(|module_rule| {
        module_rule.uses.iter().map(Box::as_ref).rev()
      })
      .collect::<Vec<_>>();

    Ok(resolved_loaders)
  }

  // FIXME: this function is duplicated with the above one, will be fixed later.
  pub fn calculate_module_type(
    &self,
    resource_data: &ResourceData,
    default_module_type: Option<ModuleType>,
  ) -> Result<ModuleType> {
    // Progressive module type resolution:
    // Stage 1: maintain the resolution logic via file extension
    // TODO: Stage 2:
    //           1. remove all extension based module type resolution, and let `module.rules[number].type` to handle this(everything is based on its config)
    //           2. set default module type to `Js`, it equals to `javascript/auto` in webpack.
    let mut resolved_module_type = default_module_type;

    self
      .context
      .options
      .module
      .rules
      .iter()
      .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
        if let Some(func) = &module_rule.func__ {
          match func(resource_data) {
            Ok(result) => {
              if result {
                return Some(Ok(module_rule));
              }

              return None
            },
            Err(e) => {
              return Some(Err(e.into()))
            }
          }
        }

        // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
        // See: https://webpack.js.org/configuration/module/#ruletest
        if let Some(test_rule) = &module_rule.test && test_rule.is_match(&resource_data.resource) {
          return Some(Ok(module_rule));
        } else if let Some(resource_rule) = &module_rule.resource && resource_rule.is_match(&resource_data.resource) {
          return Some(Ok(module_rule));
        }

        if let Some(resource_query_rule) = &module_rule.resource_query && let Some(resource_query) = &resource_data.resource_query && resource_query_rule.is_match(resource_query) {
          return Some(Ok(module_rule));
        }


        None
      })
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .for_each(|module_rule| {
        if module_rule.module_type.is_some() {
          resolved_module_type = module_rule.module_type;
        };

      });

    resolved_module_type.ok_or_else(|| {
        Error::InternalError(format!(
          "Unable to determine the module type of {}. Make sure to specify the `type` property in the module rule.",
          resource_data.resource
        ))
      },
    )
  }

  pub async fn resolve_module(
    &mut self,
    tx: &UnboundedSender<Msg>,
  ) -> Result<Option<(Vec<NormalModuleFactory>, ModuleGraphModule)>> {
    // TODO: caching in resolve, align to webpack's external module
    // Here is the corresponding create function in webpack, but instead of using hooks we use procedural functions
    let result = self
      .plugin_driver
      .read()
      .await
      .factorize_and_build(
        FactorizeAndBuildArgs {
          dependency: &self.dependency,
          plugin_driver: &self.plugin_driver,
        },
        &mut self.context,
      )
      .await?;
    let (uri, mut module) = if let Some(module) = result {
      let (uri, module) = module;
      tx.send(Msg::DependencyReference(
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
    } else if let Some(re) = self.factorize_normal_module(tx).await? {
      re
    } else {
      return Ok(None);
    };

    // scan deps

    let build_result = module
      .build(BuildContext {
        loader_runner_runner: &self.loader_runner_runner,
        resolved_loaders: self.calculate_loaders(module.resource_resolved_data())?,
        compiler_options: &self.context.options,
      })
      .await?;

    let (build_result, diagnostics) = build_result.split_into_parts();
    self.add_diagnostics(diagnostics);

    let deps = build_result
      .dependencies
      .into_iter()
      .map(|dep| Dependency {
        importer: Some(uri.clone()),
        detail: dep,
      })
      .collect::<Vec<_>>();

    tracing::trace!("get deps {:?}", deps);
    let next_tasks: Vec<NormalModuleFactory> = deps
      .iter()
      .map(|dep| {
        NormalModuleFactory::new(
          NormalModuleFactoryContext {
            module_name: None,
            module_type: None,
            ..self.context.clone()
          },
          dep.clone(),
          self.plugin_driver.clone(),
          self.loader_runner_runner.clone(),
        )
      })
      .collect();

    let resolved_module = ModuleGraphModule::new(
      self.context.module_name.clone(),
      Path::new("./")
        .join(
          Path::new(uri.as_str())
            .relative(self.plugin_driver.read().await.options.context.as_str()),
        )
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

    Ok(Some((next_tasks, resolved_module)))
  }

  // fn fork(&self, dep: Dependency) {
  //   let task = NormalModuleFactory::new(
  //     NormalModuleFactoryContext {
  //       module_name: None,
  //       module_type: None,
  //       ..self.context.clone()
  //     },
  //     dep,
  //     self.plugin_driver.clone(),
  //     self.loader_runner_runner.clone(),
  //   );

  //   tokio::task::spawn(async move {
  //     task.run().await;
  //   });
  // }
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

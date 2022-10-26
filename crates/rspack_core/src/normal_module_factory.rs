use std::{
  path::{Path, PathBuf},
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};

use sugar_path::PathSugar;
use swc_common::Span;
use tokio::sync::mpsc::UnboundedSender;

use rspack_error::{Diagnostic, Error, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::{
  parse_to_url, resolve, CompilerOptions, FactorizeAndBuildArgs, ModuleGraphModule,
  ModuleIdentifier, ModuleRule, ModuleType, Msg, NormalModule, ResolveArgs, ResolveResult,
  ResourceData, SharedPluginDriver, DEPENDENCY_ID,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Dependency {
  /// Uri of importer module
  pub importer: Option<String>,
  pub parent_module_identifier: Option<ModuleIdentifier>,
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

pub type FactorizeResult = Option<(
  ModuleGraphModule,
  NormalModule,
  Option<ModuleIdentifier>,
  u32,
)>;

#[derive(Debug)]
pub struct NormalModuleFactory {
  context: NormalModuleFactoryContext,
  dependency: Dependency,
  tx: UnboundedSender<Msg>,
  plugin_driver: SharedPluginDriver,
  diagnostic: Vec<Diagnostic>,
}

impl NormalModuleFactory {
  pub fn new(
    context: NormalModuleFactoryContext,
    dependency: Dependency,
    tx: UnboundedSender<Msg>,
    plugin_driver: SharedPluginDriver,
  ) -> Self {
    context.active_task_count.fetch_add(1, Ordering::SeqCst);

    Self {
      context,
      dependency,
      tx,
      plugin_driver,
      diagnostic: vec![],
    }
  }
  #[instrument(name = "normal_module_factory:create")]
  pub async fn create(mut self) {
    match self.factorize().await {
      Ok(maybe_module) => {
        if let Some((mgm, module, original_module_identifier, dependency_id)) = maybe_module {
          let diagnostic = std::mem::take(&mut self.diagnostic);
          if let Err(err) = self.tx.send(Msg::ModuleCreated(TWithDiagnosticArray::new(
            Box::new((
              mgm,
              module,
              original_module_identifier,
              dependency_id,
              // FIXME: redundant
              self.dependency.clone(),
            )),
            diagnostic,
          ))) {
            self
              .context
              .active_task_count
              .fetch_sub(1, Ordering::SeqCst);
            tracing::trace!("fail to send msg {:?}", err)
          }
        } else if let Err(err) = self.tx.send(Msg::ModuleCreationCanceled) {
          self
            .context
            .active_task_count
            .fetch_sub(1, Ordering::SeqCst);
          tracing::trace!("fail to send msg {:?}", err)
        }
      }
      Err(err) => {
        // If build error message is failed to send, then we should manually decrease the active task count
        // Otherwise, it will be gracefully handled by the error message handler.
        if let Err(err) = self.tx.send(Msg::ModuleCreationErrorEncountered(err)) {
          self
            .context
            .active_task_count
            .fetch_sub(1, Ordering::SeqCst);
          tracing::trace!("fail to send msg {:?}", err)
        }
      }
    }
  }

  pub fn add_diagnostic<T: Into<Diagnostic>>(&mut self, diagnostic: T) {
    self.diagnostic.push(diagnostic.into());
  }

  pub fn add_diagnostics<T: IntoIterator<Item = Diagnostic>>(&mut self, diagnostic: T) {
    self.diagnostic.extend(diagnostic);
  }

  pub fn send(&self, msg: Msg) {
    if let Err(err) = self.tx.send(msg) {
      tracing::trace!("fail to send msg {:?}", err)
    }
  }

  pub fn calculate_module_type_by_uri(&self, uri: &str) -> Option<ModuleType> {
    // todo currently unreachable module types are temporarily unified with their importers
    let url = parse_to_url(uri);
    debug_assert_eq!(url.scheme().map(|item| item.as_str()), Some("specifier"));
    resolve_module_type_by_uri(PathBuf::from(url.path().as_str()))
  }
  #[instrument(name = "normal_module_factory:factory_normal_module")]
  pub async fn factorize_normal_module(&mut self) -> Result<Option<(String, NormalModule, u32)>> {
    let importer = self.dependency.importer.as_deref();
    let specifier = self.dependency.detail.specifier.as_str();
    let kind = self.dependency.detail.kind;
    let resource_data = match resolve(
      ResolveArgs {
        importer,
        specifier,
        kind,
        span: self.dependency.detail.span,
      },
      &self.plugin_driver,
      &mut self.context,
    )
    .await?
    {
      ResolveResult::Info(info) => {
        let uri = info.join();
        ResourceData {
          resource: uri,
          resource_path: info.path.to_string_lossy().to_string(),
          resource_query: (!info.query.is_empty()).then_some(info.query),
          resource_fragment: (!info.fragment.is_empty()).then_some(info.fragment),
          ignored: false,
        }
      }
      ResolveResult::Ignored => {
        // TODO: return rawModule is a better choice.

        // TODO: Duplicate with the head code in the `resolve` function, should remove it.
        let importer = if let Some(importer) = importer {
          Path::new(importer)
            .parent()
            .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
            .to_path_buf()
        } else {
          let context = self.context.options.context.as_str();
          Path::new(context).to_path_buf()
        };
        // ----

        // TODO: just for identifier tag. should removed after Module::identifier
        let uri = format!("{}/{}", importer.display(), specifier);
        ResourceData {
          resource: uri.clone(),
          resource_path: uri,
          resource_query: None,
          resource_fragment: None,
          ignored: true,
        }
      }
    };

    let uri = resource_data.resource.clone();
    tracing::trace!("resolved uri {:?}", uri);

    if self.context.module_type.is_none() {
      self.context.module_type = self.calculate_module_type_by_uri(&uri);
    }

    let dependency_id = DEPENDENCY_ID.fetch_add(1, Ordering::Relaxed);

    debug_assert!(
      // We don't need any guarantees other than the atomicity here.
      self.context.active_task_count.load(Ordering::Relaxed) > 0,
      "Failed as the receiver end has already been dropped."
    );

    self
      .tx
      .send(Msg::DependencyReference(
        (self.dependency.clone(), dependency_id),
        uri.clone(),
      ))
      .map_err(|_| {
        Error::InternalError(format!(
          "Failed to resolve dependency {:?}",
          self.dependency
        ))
      })?;

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

    Ok(Some((uri, normal_module, dependency_id)))
  }

  pub fn calculate_module_type(
    &self,
    resource_data: &ResourceData,
    default_module_type: Option<ModuleType>,
  ) -> Result<ModuleType> {
    // TODO: should removed after `RawModule`.
    if resource_data.ignored {
      return Ok(ModuleType::Js);
    }
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

  #[instrument(name = "normal_module_factory:factorize")]
  pub async fn factorize(&mut self) -> Result<FactorizeResult> {
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

    let (uri, module, dependency_id) = if let Some(module) = result {
      // module
      let (uri, module) = module;
      // TODO: remove this
      let dependency_id = DEPENDENCY_ID.fetch_add(1, Ordering::Relaxed);

      debug_assert!(
        // We don't need any guarantees other than the atomicity here.
        self.context.active_task_count.load(Ordering::Relaxed) > 0,
        "Failed as the receiver end has already been dropped."
      );

      self
        .tx
        .send(Msg::DependencyReference(
          (self.dependency.clone(), dependency_id),
          uri.clone(),
        ))
        .map_err(|_| {
          Error::InternalError(format!(
            "Failed to resolve dependency {:?}",
            self.dependency
          ))
        })?;
      (uri, module, dependency_id)
    } else if let Some(result) = self.factorize_normal_module().await? {
      result
    } else {
      return Ok(None);
    };

    let mgm = ModuleGraphModule::new(
      self.context.module_name.clone(),
      Path::new("./")
        .join(
          Path::new(uri.as_str())
            .relative(self.plugin_driver.read().await.options.context.as_str()),
        )
        .to_string_lossy()
        .to_string(),
      uri,
      module.identifier(),
      vec![],
      self
        .context
        .module_type
        .ok_or_else(|| Error::InternalError("source type is empty".to_string()))?,
    );

    Ok(Some((
      mgm,
      module,
      self.dependency.parent_module_identifier.clone(),
      dependency_id,
    )))
  }

  // fn fork(&self, dep: Dependency) {
  //   let normal_module_factory = NormalModuleFactory::new(
  //     NormalModuleFactoryContext {
  //       module_name: None,
  //       module_type: None,
  //       ..self.context.clone()
  //     },
  //     dep,
  //     self.tx.clone(),
  //     self.plugin_driver.clone(),
  //   );

  //   tokio::task::spawn(async move {
  //     normal_module_factory.create().await;
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
  pub(crate) active_task_count: Arc<AtomicU32>,
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

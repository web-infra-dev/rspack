use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{internal_error, Error, Result};
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::Span;
use tracing::instrument;

use crate::{
  cache::Cache, module_rule_matcher, resolve, BoxModule, CompilerOptions, Dependency,
  FactorizeArgs, Identifiable, ModuleArgs, ModuleDependency, ModuleExt, ModuleIdentifier,
  ModuleRule, ModuleType, NormalModule, RawModule, Resolve, ResolveArgs, ResolveResult,
  ResourceData, SharedPluginDriver,
};

// #[derive(Debug, Hash, PartialEq, Eq, Clone)]
// pub struct Dependency {
//   /// Parent module identifier (Can be used to locate its parent module in module graph)
//   pub parent_module_identifier: Option<ModuleIdentifier>,
//   pub detail: ModuleDependency,
// }

// #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
// pub enum ResolveKind {
//   Entry,
//   Import,
//   Require,
//   DynamicImport,
//   AtImport,
//   UrlToken,
//   ModuleHotAccept,
// }

#[derive(Debug)]
pub struct FactorizeResult {
  pub module: BoxModule,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
}

impl FactorizeResult {
  pub fn new(module: BoxModule) -> Self {
    Self {
      module,
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
    }
  }

  pub fn file_dependency(mut self, file: PathBuf) -> Self {
    self.file_dependencies.insert(file);
    self
  }

  pub fn file_dependencies(mut self, files: impl IntoIterator<Item = PathBuf>) -> Self {
    self.file_dependencies.extend(files);
    self
  }

  pub fn context_dependency(mut self, context: PathBuf) -> Self {
    self.context_dependencies.insert(context);
    self
  }

  pub fn context_dependencies(mut self, contexts: impl IntoIterator<Item = PathBuf>) -> Self {
    self.context_dependencies.extend(contexts);
    self
  }

  pub fn missing_dependency(mut self, missing: PathBuf) -> Self {
    self.missing_dependencies.insert(missing);
    self
  }

  pub fn missing_dependencies(mut self, missings: impl IntoIterator<Item = PathBuf>) -> Self {
    self.missing_dependencies.extend(missings);
    self
  }
}

#[derive(Debug)]
pub struct NormalModuleFactory {
  context: NormalModuleFactoryContext,
  dependency: Box<dyn ModuleDependency>,
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
}

impl NormalModuleFactory {
  pub fn new(
    context: NormalModuleFactoryContext,
    dependency: Box<dyn ModuleDependency>,
    plugin_driver: SharedPluginDriver,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      context,
      dependency,
      plugin_driver,
      cache,
    }
  }

  /// set `is_entry` true if you are trying to create a new module factory with a module identifier which is an entry
  #[instrument(name = "normal_module_factory:create", skip_all)]
  pub async fn create(
    mut self,
    resolve_options: Option<Resolve>,
  ) -> Result<(FactorizeResult, NormalModuleFactoryContext)> {
    Ok((self.factorize(resolve_options).await?, self.context))
  }

  pub fn calculate_module_type_by_resource(
    &self,
    resource_data: &ResourceData,
  ) -> Option<ModuleType> {
    // todo currently unreachable module types are temporarily unified with their importers
    resolve_module_type_by_uri(&resource_data.resource_path)
  }

  #[instrument(name = "normal_module_factory:factory_normal_module", skip_all)]
  pub async fn factorize_normal_module(
    &mut self,
    resolve_options: Option<Resolve>,
  ) -> Result<Option<FactorizeResult>> {
    // TODO: `importer` should use `NormalModule::context || options.context`;
    let importer = self
      .dependency
      .parent_module_identifier()
      .map(|i| i.as_str());
    let specifier = self.dependency.request();
    if should_skip_resolve(specifier) {
      return Ok(None);
    }
    let resolve_args = ResolveArgs {
      importer,
      specifier,
      dependency_type: self.dependency.dependency_type(),
      dependency_category: self.dependency.category(),
      span: self.dependency.span().cloned(),
      resolve_options,
    };
    let plugin_driver = self.plugin_driver.clone();
    let resource_data = self
      .cache
      .resolve_module_occasion
      .use_cache(resolve_args, |args| resolve(args, &plugin_driver))
      .await?;
    let resource_data = match resource_data {
      ResolveResult::Info(info) => {
        let uri = info.join();
        ResourceData {
          resource: uri,
          resource_path: info.path,
          resource_query: (!info.query.is_empty()).then_some(info.query),
          resource_fragment: (!info.fragment.is_empty()).then_some(info.fragment),
        }
      }
      ResolveResult::Ignored => {
        // TODO: Duplicate with the head code in the `resolve` function, should remove it.
        let importer = if let Some(importer) = importer {
          Path::new(importer)
            .parent()
            .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
            .to_path_buf()
        } else {
          PathBuf::from(self.context.options.context.as_path())
        };
        // ----

        // TODO: just for identifier tag. should removed after Module::identifier
        let uri = format!("{}/{}", importer.display(), specifier);

        let module_identifier = ModuleIdentifier::from(format!("ignored|{uri}"));
        let raw_module = RawModule::new(
          "/* (ignored) */".to_owned(),
          module_identifier,
          format!("{uri} (ignored)"),
          Default::default(),
        )
        .boxed();

        self.context.module_type = Some(*raw_module.module_type());

        return Ok(Some(FactorizeResult::new(raw_module)));
      }
    };

    let uri = resource_data.resource.clone();
    tracing::trace!("resolved uri {:?}", uri);

    if self.context.module_type.is_none() {
      self.context.module_type = self.calculate_module_type_by_resource(&resource_data);
    }

    let resolved_module_rules = self.calculate_module_rules(&resource_data)?;
    let resolved_module_type = self.calculate_module_type(
      &resolved_module_rules,
      &resource_data,
      self.context.module_type,
    )?;
    let resolved_resolve_options = self.calculate_resolve_options(&resolved_module_rules);

    let resolved_parser_and_generator = self
      .plugin_driver
      .read()
      .await
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        Error::InternalError(internal_error!(format!(
          "Parser and generator builder for module type {resolved_module_type:?} is not registered"
        )))
      })?();

    self.context.module_type = Some(resolved_module_type);

    let file_dependency = resource_data.resource_path.clone();

    let normal_module = NormalModule::new(
      uri.clone(),
      uri.clone(),
      self.dependency.request().to_owned(),
      resolved_module_type,
      resolved_parser_and_generator,
      resource_data,
      resolved_resolve_options,
      self.context.options.clone(),
    );

    let module = if let Some(module) = self
      .plugin_driver
      .read()
      .await
      .module(ModuleArgs {
        dependency_type: *self.dependency.dependency_type(),
        indentfiler: normal_module.identifier(),
        lazy_visit_modules: self.context.lazy_visit_modules.clone(),
      })
      .await?
    {
      module
    } else {
      Box::new(normal_module)
    };

    Ok(Some(
      FactorizeResult::new(module).file_dependency(file_dependency),
    ))
  }

  fn calculate_module_rules(&self, resource_data: &ResourceData) -> Result<Vec<&ModuleRule>> {
    self
      .context
      .options
      .module
      .rules
      .iter()
      .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
        match module_rule_matcher(module_rule, resource_data) {
          Ok(val) => val.then_some(Ok(module_rule)),
          Err(err) => Some(Err(err)),
        }
      })
      .collect::<Result<Vec<_>>>()
  }

  fn calculate_resolve_options(&self, module_rules: &[&ModuleRule]) -> Option<Resolve> {
    let mut resolved = None;
    module_rules.iter().for_each(|rule| {
      if let Some(resolve) = rule.resolve.to_owned() {
        resolved = Some(resolve);
      }
    });
    resolved
  }

  pub fn calculate_module_type(
    &self,
    module_rules: &[&ModuleRule],
    resource_data: &ResourceData,
    default_module_type: Option<ModuleType>,
  ) -> Result<ModuleType> {
    // Progressive module type resolution:
    // Stage 1: maintain the resolution logic via file extension
    // TODO: Stage 2:
    //           1. remove all extension based module type resolution, and let `module.rules[number].type` to handle this(everything is based on its config)
    //           2. set default module type to `Js`, it equals to `javascript/auto` in webpack.
    let mut resolved_module_type = default_module_type;

    module_rules.iter().for_each(|module_rule| {
      if module_rule.r#type.is_some() {
        resolved_module_type = module_rule.r#type;
      };
    });

    resolved_module_type.ok_or_else(|| {
        Error::InternalError(internal_error!(format!(
          "Unable to determine the module type of {}. Make sure to specify the `type` property in the module rule.",
          resource_data.resource
        )))
      },
    )
  }

  #[instrument(name = "normal_module_factory:factorize", skip_all)]
  pub async fn factorize(&mut self, resolve_options: Option<Resolve>) -> Result<FactorizeResult> {
    let result = self
      .plugin_driver
      .read()
      .await
      .factorize(
        FactorizeArgs {
          dependency: &*self.dependency,
          plugin_driver: &self.plugin_driver,
        },
        &mut self.context,
      )
      .await?;

    if let Some(result) = result {
      self.context.module_type = Some(*result.module.module_type());
      return Ok(result);
    }

    if let Some(result) = self.factorize_normal_module(resolve_options).await? {
      return Ok(result);
    }

    Err(Error::InternalError(internal_error!(
      "Failed to factorize module, neither hook nor factorize method returns".to_owned()
    )))
  }
}

pub fn should_skip_resolve(s: &str) -> bool {
  s.starts_with("data:")
    || s.starts_with("http://")
    || s.starts_with("https://")
    || s.starts_with("//")
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
  pub module_type: Option<ModuleType>,
  pub side_effects: Option<bool>,
  pub options: Arc<CompilerOptions>,
  pub lazy_visit_modules: std::collections::HashSet<String>,
}

// #[derive(Debug, Clone, Eq)]
// pub struct ModuleDependency {
//   pub specifier: String,
//   /// `./a.js` in `import './a.js'` is specifier
//   pub kind: ResolveKind,
//   pub span: Option<ErrorSpan>,
// }

// /// # WARNING
// /// Don't update the manual implementation of `Hash` of [ModuleDependency]
// /// Current implementation strong rely on the field of `specifier` and `kind`
// impl std::hash::Hash for ModuleDependency {
//   fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//     self.specifier.hash(state);
//     self.kind.hash(state);
//   }
// }
// /// # WARNING
// /// Don't update the manual implementation of `PartialEq` of [ModuleDependency]
// /// Current implementation strong rely on the field of `specifier` and `kind`
// impl PartialEq for ModuleDependency {
//   fn eq(&self, other: &Self) -> bool {
//     self.specifier == other.specifier && self.kind == other.kind
//   }
// }

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
      start: span.lo.0.saturating_sub(1),
      end: span.hi.0.saturating_sub(1),
    }
  }
}

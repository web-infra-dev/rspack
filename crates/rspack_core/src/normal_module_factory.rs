use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::Identifiable;
use swc_core::common::Span;

use crate::{
  cache::Cache, module_rule_matcher, resolve, AssetGeneratorOptions, AssetParserOptions,
  CompilerOptions, Dependency, FactorizeArgs, MissingModule, ModuleArgs, ModuleDependency,
  ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier,
  ModuleRule, ModuleType, NormalModule, RawModule, Resolve, ResolveArgs, ResolveError,
  ResolveResult, ResourceData, SharedPluginDriver,
};

#[derive(Debug)]
pub struct NormalModuleFactory {
  context: NormalModuleFactoryContext,
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
}

#[async_trait::async_trait]
impl ModuleFactory for NormalModuleFactory {
  async fn create(
    mut self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    Ok(self.factorize(data).await?)
  }
}

impl NormalModuleFactory {
  pub fn new(
    context: NormalModuleFactoryContext,
    plugin_driver: SharedPluginDriver,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      context,
      plugin_driver,
      cache,
    }
  }

  pub async fn factorize_normal_module(
    &mut self,
    data: ModuleFactoryCreateData,
  ) -> Result<Option<TWithDiagnosticArray<ModuleFactoryResult>>> {
    let importer = self.context.original_resource_path.as_ref();
    let importer_with_context = if let Some(importer) = importer {
      Path::new(importer)
        .parent()
        .ok_or_else(|| anyhow::format_err!("parent() failed for {:?}", importer))?
        .to_path_buf()
    } else {
      PathBuf::from(self.context.options.context.as_path())
    };
    let specifier = data.dependency.request();
    if should_skip_resolve(specifier) {
      return Ok(None);
    }

    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();

    let resolve_args = ResolveArgs {
      importer,
      context: data.context,
      specifier,
      dependency_type: data.dependency.dependency_type(),
      dependency_category: data.dependency.category(),
      span: data.dependency.span().cloned(),
      resolve_options: data.resolve_options,
      resolve_to_context: false,
      file_dependencies: &mut file_dependencies,
      missing_dependencies: &mut missing_dependencies,
    };
    let plugin_driver = &self.plugin_driver;
    let resource_data = self
      .cache
      .resolve_module_occasion
      .use_cache(resolve_args, |args| resolve(args, plugin_driver))
      .await;
    let resource_data = match resource_data {
      Ok(ResolveResult::Resource(resource)) => {
        let uri = resource.join().display().to_string();
        ResourceData {
          resource: uri,
          resource_path: resource.path,
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          resource_description: resource.description,
        }
      }
      Ok(ResolveResult::Ignored) => {
        let ident = format!("{}/{}", importer_with_context.display(), specifier);
        let module_identifier = ModuleIdentifier::from(format!("ignored|{ident}"));

        let raw_module = RawModule::new(
          "/* (ignored) */".to_owned(),
          module_identifier,
          format!("{ident} (ignored)"),
          Default::default(),
        )
        .boxed();
        self.context.module_type = Some(*raw_module.module_type());

        return Ok(Some(
          ModuleFactoryResult::new(raw_module).with_empty_diagnostic(),
        ));
      }
      Err(ResolveError(runtime_error, internal_error)) => {
        let ident = format!("{}{specifier}", importer_with_context.display());
        let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

        let missing_module = MissingModule::new(
          module_identifier,
          format!("{ident} (missing)"),
          runtime_error,
        )
        .boxed();
        self.context.module_type = Some(*missing_module.module_type());
        return Ok(Some(
          ModuleFactoryResult::new(missing_module).with_diagnostic(internal_error.into()),
        ));
      }
    };

    let loaders = self
      .context
      .options
      .module
      .rules
      .iter()
      .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
        match module_rule_matcher(
          module_rule,
          &resource_data,
          importer.map(|i| i.to_string_lossy()).as_deref(),
        ) {
          Ok(val) => val.map(Ok),
          Err(err) => Some(Err(err)),
        }
      })
      .collect::<Result<Vec<_>>>()?;

    let loaders = loaders
      .into_iter()
      .flat_map(|module_rule| module_rule.r#use.iter().cloned().rev())
      .collect::<Vec<_>>();

    let request = if !loaders.is_empty() {
      let s = loaders
        .iter()
        .map(|i| i.name())
        .collect::<Vec<_>>()
        .join("!");
      format!("{s}!{}", resource_data.resource)
    } else {
      resource_data.resource.clone()
    };
    let user_request = resource_data.resource.clone();
    tracing::trace!("resolved uri {:?}", request);

    let file_dependency = resource_data.resource_path.clone();

    let resolved_module_rules = self.calculate_module_rules(&resource_data)?;
    let resolved_module_type =
      self.calculate_module_type(&resolved_module_rules, self.context.module_type);
    let resolved_resolve_options = self.calculate_resolve_options(&resolved_module_rules);
    let (resolved_parser_options, resolved_generator_options) =
      self.calculate_parser_and_generator_options(&resolved_module_rules);

    let resolved_parser_and_generator = self
      .plugin_driver
      .read()
      .await
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        internal_error!(
          "Parser and generator builder for module type {resolved_module_type:?} is not registered"
        )
      })?();

    self.context.module_type = Some(resolved_module_type);

    let normal_module = NormalModule::new(
      request,
      user_request,
      data.dependency.request().to_owned(),
      resolved_module_type,
      resolved_parser_and_generator,
      resolved_parser_options,
      resolved_generator_options,
      resource_data,
      resolved_resolve_options,
      loaders,
      self.context.options.clone(),
    );

    let module = if let Some(module) = self
      .plugin_driver
      .read()
      .await
      .module(ModuleArgs {
        dependency_type: *data.dependency.dependency_type(),
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
      ModuleFactoryResult::new(module)
        .file_dependency(file_dependency)
        .file_dependencies(file_dependencies)
        .missing_dependencies(missing_dependencies)
        .with_empty_diagnostic(),
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
        match module_rule_matcher(module_rule, resource_data, self.context.issuer.as_deref()) {
          Ok(val) => val.map(Ok),
          Err(err) => Some(Err(err)),
        }
      })
      .collect::<Result<Vec<_>>>()
  }

  fn calculate_resolve_options(&self, module_rules: &[&ModuleRule]) -> Option<Resolve> {
    let mut resolved = None;
    module_rules.iter().for_each(|rule| {
      if let Some(resolve) = rule.resolve.as_ref() {
        resolved = Some(resolve.to_owned());
      }
    });
    resolved
  }

  fn calculate_parser_and_generator_options(
    &self,
    module_rules: &[&ModuleRule],
  ) -> (Option<AssetParserOptions>, Option<AssetGeneratorOptions>) {
    let mut resolved_parser: Option<AssetParserOptions> = None;
    let mut resolved_generator: Option<AssetGeneratorOptions> = None;

    module_rules.iter().for_each(|rule| {
      // TODO: should deep merge
      if let Some(parser) = rule.parser.as_ref() {
        resolved_parser = Some(parser.to_owned());
      }
      if let Some(generator) = rule.generator.as_ref() {
        resolved_generator = Some(generator.to_owned());
      }
    });

    (resolved_parser, resolved_generator)
  }

  pub fn calculate_module_type(
    &self,
    module_rules: &[&ModuleRule],
    default_module_type: Option<ModuleType>,
  ) -> ModuleType {
    let mut resolved_module_type = default_module_type.unwrap_or(ModuleType::Js);

    module_rules.iter().for_each(|module_rule| {
      if let Some(module_type) = module_rule.r#type {
        resolved_module_type = module_type;
      };
    });

    resolved_module_type
  }

  pub async fn factorize(
    &mut self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let result = self
      .plugin_driver
      .read()
      .await
      .factorize(
        FactorizeArgs {
          dependency: &*data.dependency,
          plugin_driver: &self.plugin_driver,
        },
        &mut self.context,
      )
      .await?;

    if let Some(result) = result {
      self.context.module_type = Some(*result.module.module_type());
      return Ok(result.with_empty_diagnostic());
    }

    if let Some(result) = self.factorize_normal_module(data).await? {
      return Ok(result);
    }

    Err(internal_error!(
      "Failed to factorize module, neither hook nor factorize method returns"
    ))
  }
}

pub fn should_skip_resolve(s: &str) -> bool {
  s.starts_with("data:")
    || s.starts_with("http://")
    || s.starts_with("https://")
    || s.starts_with("//")
}

#[derive(Debug, Clone)]
pub struct NormalModuleFactoryContext {
  pub original_resource_path: Option<PathBuf>,
  pub module_type: Option<ModuleType>,
  pub side_effects: Option<bool>,
  pub options: Arc<CompilerOptions>,
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub issuer: Option<String>,
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
      start: span.lo.0.saturating_sub(1),
      end: span.hi.0.saturating_sub(1),
    }
  }
}

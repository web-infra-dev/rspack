use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::Identifiable;
use sugar_path::AsPath;
use swc_core::common::Span;

use crate::{
  cache::Cache, module_rule_matcher, resolve, AssetGeneratorOptions, AssetParserOptions, BoxLoader,
  CompilerOptions, Dependency, DependencyCategory, DependencyType, FactorizeArgs, FactoryMeta,
  MissingModule, ModuleArgs, ModuleDependency, ModuleExt, ModuleFactory, ModuleFactoryCreateData,
  ModuleFactoryResult, ModuleIdentifier, ModuleRule, ModuleRuleEnforce, ModuleType, NormalModule,
  NormalModuleFactoryResolveForSchemeArgs, RawModule, Resolve, ResolveArgs, ResolveError,
  ResolveOptionsWithDependencyType, ResolveResult, ResolverFactory, ResourceData,
  SharedPluginDriver,
};

#[derive(Debug)]
pub struct NormalModuleFactory {
  context: NormalModuleFactoryContext,
  resolver_factory: Arc<ResolverFactory>,
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
    resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      context,
      resolver_factory,
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

    let mut resolve_args = ResolveArgs {
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

    let scheme = url::Url::parse(specifier)
      .map(|url| url.scheme().to_string())
      .ok();
    let plugin_driver = &self.plugin_driver;
    let mut inline_loaders: Vec<BoxLoader> = vec![];
    let mut no_pre_auto_loaders = false;
    let mut no_auto_loaders = false;
    let mut no_pre_post_auto_loaders = false;

    // with scheme, windows absolute path is considered scheme by `url`
    let resource_data = if let Some(scheme) = scheme && !Path::is_absolute(Path::new(specifier)) {
      let data = plugin_driver
        .read()
        .await
        .normal_module_factory_resolve_for_scheme(NormalModuleFactoryResolveForSchemeArgs {
          resource: ResourceData {
            resource: specifier.to_string(),
            resource_description: None,
            resource_fragment: None,
            resource_query: None,
            resource_path: "".into(),
          },
          scheme,
        })
        .await;
      match data {
        Ok(Some(data)) => data,
        Ok(None) => {
          let ident = format!("{}{specifier}", importer_with_context.display());
          let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

          let missing_module = MissingModule::new(
            module_identifier,
            format!("{ident} (missing)"),
            format!("Failed to resolve {specifier}"),
          )
          .boxed();
          self.context.module_type = Some(*missing_module.module_type());
          return Ok(Some(
            ModuleFactoryResult::new(missing_module).with_empty_diagnostic(),
          ));
        }
        Err(err) => {
          return Err(err);
        }
      }
    } else {
      {
        let mut request = specifier.chars();
        let first_char = request.next();
        let second_char = request.next();
        // See: https://webpack.js.org/concepts/loaders/#inline
        no_pre_auto_loaders = matches!(first_char, Some('-')) && matches!(second_char, Some('!'));
        no_auto_loaders = no_pre_auto_loaders || matches!(first_char, Some('!'));
        no_pre_post_auto_loaders = matches!(first_char, Some('!')) && matches!(second_char, Some('!'));

        let mut raw_elements = {
          let s = match specifier.char_indices().nth({
            if no_pre_auto_loaders || no_pre_post_auto_loaders {
              2
            } else if no_auto_loaders {
              1
            } else {
              0
            }
          }) {
            Some((pos, _)) => {
              &specifier[pos..]
            },
            None=> {
              let dependency = data.dependency;
              unreachable!("Invalid dependency: {dependency:?}")
            }
          };
          s.split('!').filter(|item| !item.is_empty()).collect::<Vec<_>>()
        };
        resolve_args.specifier = raw_elements.pop().ok_or_else(|| {
          let s = resolve_args.specifier;
          internal_error!("Invalid request: {s}")
        })?;

        let loader_resolver = self.resolver_factory.get(ResolveOptionsWithDependencyType {
          resolve_options: resolve_args.resolve_options.clone(),
          resolve_to_context: false,
          dependency_type: DependencyType::CjsRequire,
          dependency_category: DependencyCategory::CommonJS,
        });

        let plugin_driver = self.plugin_driver.read().await;
        for element in raw_elements {
          let importer = resolve_args.importer.map(|i| i.display().to_string());
          let res = plugin_driver.resolve_loader(
            &self.context.options,
            {
              if let Some(context) = &resolve_args.context {
                context.as_path()
              } else if let Some(i) = importer.as_ref() {
                {
                  // TODO: delete this fn after use `normalModule.context` rather than `importer`
                  if let Some(index) = i.find('?') {
                    Path::new(&i[0..index])
                  } else {
                    Path::new(i)
                  }
                }
                .parent()
                .ok_or_else(|| internal_error!("parent() failed for {:?}", importer))?
              } else {
                &plugin_driver.options.context
              }
            },
            &loader_resolver,
            element
            )
            .await?.ok_or_else(|| {
              internal_error!("Loader expected")
            })?;
          inline_loaders.push(res);
        }
      }

      // default resolve
      let resource_data = self
        .cache
        .resolve_module_occasion
        .use_cache(resolve_args, |args| resolve(args, plugin_driver))
        .await;
      match resource_data {
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
      }
    };
    //TODO: with contextScheme
    let resolved_module_rules = self
      .calculate_module_rules(&resource_data, data.dependency.category())
      .await?;

    let user_request = if !inline_loaders.is_empty() {
      let s = inline_loaders
        .iter()
        .map(|i| i.identifier().as_str())
        .collect::<Vec<_>>()
        .join("!");
      format!("{s}!{}", resource_data.resource)
    } else {
      resource_data.resource.clone()
    };

    // TODO: move loader resolver to rust
    let loaders: Vec<BoxLoader> = {
      let mut pre_loaders: Vec<BoxLoader> = vec![];
      let mut post_loaders: Vec<BoxLoader> = vec![];
      let mut normal_loaders: Vec<BoxLoader> = vec![];

      for rule in &resolved_module_rules {
        match rule.enforce {
          ModuleRuleEnforce::Pre => {
            if !no_pre_auto_loaders && !no_pre_post_auto_loaders {
              pre_loaders.extend_from_slice(&rule.r#use);
            }
          }
          ModuleRuleEnforce::Normal => {
            if !no_auto_loaders && !no_pre_auto_loaders {
              normal_loaders.extend_from_slice(&rule.r#use);
            }
          }
          ModuleRuleEnforce::Post => {
            if !no_pre_post_auto_loaders {
              post_loaders.extend_from_slice(&rule.r#use);
            }
          }
        }
      }

      let mut all_loaders = Vec::with_capacity(
        pre_loaders.len() + post_loaders.len() + normal_loaders.len() + inline_loaders.len(),
      );

      all_loaders.extend(post_loaders);
      all_loaders.extend(inline_loaders);
      all_loaders.extend(normal_loaders);
      all_loaders.extend(pre_loaders);

      all_loaders
    };

    let request = if !loaders.is_empty() {
      let s = loaders
        .iter()
        .map(|i| i.identifier().as_str())
        .collect::<Vec<_>>()
        .join("!");
      format!("{s}!{}", resource_data.resource)
    } else {
      resource_data.resource.clone()
    };
    tracing::trace!("resolved uri {:?}", request);

    let file_dependency = resource_data.resource_path.clone();

    let resolved_module_type =
      self.calculate_module_type(&resolved_module_rules, self.context.module_type);
    let resolved_resolve_options = self.calculate_resolve_options(&resolved_module_rules);
    let (resolved_parser_options, resolved_generator_options) =
      self.calculate_parser_and_generator_options(&resolved_module_rules);
    let factory_meta = FactoryMeta {
      side_effects: self.calculate_side_effects(&resolved_module_rules),
    };

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
        .factory_meta(factory_meta)
        .with_empty_diagnostic(),
    ))
  }

  async fn calculate_module_rules(
    &self,
    resource_data: &ResourceData,
    dependency: &DependencyCategory,
  ) -> Result<Vec<&ModuleRule>> {
    let mut rules = Vec::new();
    for rule in &self.context.options.module.rules {
      if let Some(rule) = module_rule_matcher(
        rule,
        resource_data,
        self.context.issuer.as_deref(),
        dependency,
      )
      .await?
      {
        rules.push(rule);
      }
    }
    Ok(rules)
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

  fn calculate_side_effects(&self, module_rules: &[&ModuleRule]) -> Option<bool> {
    let mut side_effects = None;
    module_rules.iter().for_each(|rule| {
      side_effects = rule.side_effects;
    });
    side_effects
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

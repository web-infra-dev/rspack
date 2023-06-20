use std::{path::Path, sync::Arc};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::{internal_error, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_identifier::Identifiable;
use rspack_loader_runner::{get_scheme, Scheme};
use sugar_path::{AsPath, SugarPath};
use swc_core::common::Span;

use crate::{
  cache::Cache,
  module_rules_matcher, parse_resource, resolve, stringify_loaders_and_resource,
  tree_shaking::visitor::{get_side_effects_from_package_json, SideEffects},
  BoxLoader, CompilerOptions, Dependency, DependencyCategory, DependencyType, FactorizeArgs,
  FactoryMeta, GeneratorOptions, MissingModule, ModuleArgs, ModuleDependency, ModuleExt,
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, ModuleRule,
  ModuleRuleEnforce, ModuleType, NormalModule, NormalModuleAfterResolveArgs,
  NormalModuleBeforeResolveArgs, ParserOptions, RawModule, Resolve, ResolveArgs, ResolveError,
  ResolveOptionsWithDependencyType, ResolveResult, ResolverFactory, ResourceData,
  ResourceParsedData, SharedPluginDriver,
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
    mut data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    if let Ok(Some(before_resolve_data)) = self.before_resolve(&mut data).await {
      return Ok(before_resolve_data);
    }
    let (factory_result, diagnostics) = self.factorize(&mut data).await?.split_into_parts();
    if let Ok(Some(after_resolve_data)) = self.after_resolve(&data, &factory_result).await {
      return Ok(after_resolve_data);
    }

    Ok(factory_result.with_diagnostic(diagnostics))
  }
}

static MATCH_RESOURCE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("^([^!]+)!=!").expect("Failed to initialize `MATCH_RESOURCE_REGEX`"));

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

  pub async fn before_resolve(
    &mut self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<TWithDiagnosticArray<ModuleFactoryResult>>> {
    // allow javascript plugin to modify args
    let mut before_resolve_args = NormalModuleBeforeResolveArgs {
      request: data.dependency.request().to_string(),
      context: data.context.to_string(),
    };
    if let Ok(Some(false)) = self
      .plugin_driver
      .before_resolve(&mut before_resolve_args)
      .await
    {
      let request_without_match_resource = data.dependency.request();
      let ident = format!("{}/{request_without_match_resource}", &data.context);
      let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

      let missing_module = MissingModule::new(
        module_identifier,
        format!("{ident} (missing)"),
        format!("Failed to resolve {request_without_match_resource}"),
      )
      .boxed();
      self.context.module_type = Some(*missing_module.module_type());
      return Ok(Some(
        ModuleFactoryResult::new(missing_module).with_empty_diagnostic(),
      ));
    }

    data.context = before_resolve_args.context.into();
    data.dependency.set_request(before_resolve_args.request);
    Ok(None)
  }

  pub async fn after_resolve(
    &mut self,
    data: &ModuleFactoryCreateData,
    factory_result: &ModuleFactoryResult,
  ) -> Result<Option<TWithDiagnosticArray<ModuleFactoryResult>>> {
    if let Ok(Some(false)) = self
      .plugin_driver
      .after_resolve(NormalModuleAfterResolveArgs {
        request: data.dependency.request(),
        context: data.context.as_ref(),
        file_dependencies: &factory_result.file_dependencies,
        context_dependencies: &factory_result.context_dependencies,
        missing_dependencies: &factory_result.missing_dependencies,
        factory_meta: &factory_result.factory_meta,
      })
      .await
    {
      let request_without_match_resource = data.dependency.request();
      let ident = format!("{}/{request_without_match_resource}", &data.context);
      let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

      let missing_module = MissingModule::new(
        module_identifier,
        format!("{ident} (missing)"),
        format!("Failed to resolve {request_without_match_resource}"),
      )
      .boxed();
      self.context.module_type = Some(*missing_module.module_type());
      return Ok(Some(
        ModuleFactoryResult::new(missing_module).with_empty_diagnostic(),
      ));
    }
    Ok(None)
  }
  pub async fn factorize_normal_module(
    &mut self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<TWithDiagnosticArray<ModuleFactoryResult>>> {
    let importer = self.context.original_module_identifier.as_ref();
    let mut request_without_match_resource = data.dependency.request();

    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();

    let scheme = get_scheme(request_without_match_resource);
    let context_scheme = get_scheme(data.context.as_ref());
    let plugin_driver = &self.plugin_driver;

    let mut match_resource_data: Option<ResourceData> = None;
    let mut inline_loaders: Vec<BoxLoader> = vec![];
    let mut no_pre_auto_loaders = false;
    let mut no_auto_loaders = false;
    let mut no_pre_post_auto_loaders = false;

    // with scheme, windows absolute path is considered scheme by `url`
    let resource_data = if scheme != Scheme::None
      && !Path::is_absolute(Path::new(request_without_match_resource))
    {
      // resource with scheme
      plugin_driver
        .normal_module_factory_resolve_for_scheme(ResourceData::new(
          request_without_match_resource.to_string(),
          "".into(),
        ))
        .await?
    }
    // TODO: resource within scheme, call resolveInScheme hook
    else {
      {
        let plugin_driver = &self.plugin_driver;
        let context = data.context.as_path();
        request_without_match_resource = {
          let match_resource_match = MATCH_RESOURCE_REGEX.captures(request_without_match_resource);
          if let Some(m) = match_resource_match {
            let mut match_resource: String = m
              .get(1)
              .expect("Should have match resource")
              .as_str()
              .to_owned();
            let mut chars = match_resource.chars();
            let first_char = chars.next();
            let second_char = chars.next();

            if matches!(first_char, Some('.'))
              && (matches!(second_char, Some('/'))
                || (matches!(second_char, Some('.')) && matches!(chars.next(), Some('/'))))
            {
              // if matchResources startsWith ../ or ./
              match_resource = context
                .join(match_resource)
                .absolutize()
                .to_string_lossy()
                .to_string();
            }

            let ResourceParsedData {
              path,
              query,
              fragment,
            } = parse_resource(&match_resource).expect("Should parse resource");
            match_resource_data = Some(
              ResourceData::new(match_resource, path)
                .query_optional(query)
                .fragment_optional(fragment),
            );

            // e.g. ./index.js!=!
            let whole_matched = m.get(0).expect("Whole matched").as_str();

            match request_without_match_resource
              .char_indices()
              .nth(whole_matched.len())
            {
              Some((pos, _)) => &request_without_match_resource[pos..],
              None => {
                unreachable!("Invalid dependency: {:?}", &data.dependency)
              }
            }
          } else {
            request_without_match_resource
          }
        };

        // dbg!(&match_resource_data);

        let mut request = request_without_match_resource.chars();
        let first_char = request.next();
        let second_char = request.next();
        // See: https://webpack.js.org/concepts/loaders/#inline
        no_pre_auto_loaders = matches!(first_char, Some('-')) && matches!(second_char, Some('!'));
        no_auto_loaders = no_pre_auto_loaders || matches!(first_char, Some('!'));
        no_pre_post_auto_loaders =
          matches!(first_char, Some('!')) && matches!(second_char, Some('!'));

        let mut raw_elements = {
          let s = match request_without_match_resource.char_indices().nth({
            if no_pre_auto_loaders || no_pre_post_auto_loaders {
              2
            } else if no_auto_loaders {
              1
            } else {
              0
            }
          }) {
            Some((pos, _)) => &request_without_match_resource[pos..],
            None => {
              let dependency = &data.dependency;
              unreachable!("Invalid dependency: {dependency:?}")
            }
          };
          s.split('!')
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
        };
        request_without_match_resource = raw_elements
          .pop()
          .ok_or_else(|| internal_error!("Invalid request: {request_without_match_resource}"))?;

        let loader_resolver = self.resolver_factory.get(ResolveOptionsWithDependencyType {
          resolve_options: data.resolve_options.clone(),
          resolve_to_context: false,
          dependency_type: DependencyType::CjsRequire,
          dependency_category: DependencyCategory::CommonJS,
        });

        for element in raw_elements {
          let res = plugin_driver
            .resolve_loader(&self.context.options, context, &loader_resolver, element)
            .await?
            .ok_or_else(|| internal_error!("Loader expected"))?;
          inline_loaders.push(res);
        }
      }
      let optional = data.dependency.get_optional();

      let resolve_args = ResolveArgs {
        importer,
        context: if context_scheme != Scheme::None {
          self.context.options.context.clone()
        } else {
          data.context.clone()
        },
        specifier: request_without_match_resource,
        dependency_type: data.dependency.dependency_type(),
        dependency_category: data.dependency.category(),
        span: data.dependency.span().cloned(),
        // take the options is safe here, because it
        // is not used in after_resolve hooks
        resolve_options: data.resolve_options.take(),
        resolve_to_context: false,
        optional,
        file_dependencies: &mut file_dependencies,
        missing_dependencies: &mut missing_dependencies,
      };

      // default resolve
      let resource_data = self
        .cache
        .resolve_module_occasion
        .use_cache(resolve_args, |args| resolve(args, plugin_driver))
        .await;
      match resource_data {
        Ok(ResolveResult::Resource(resource)) => {
          let uri = resource.join().display().to_string();
          ResourceData::new(uri, resource.path)
            .query_optional(resource.query)
            .fragment_optional(resource.fragment)
            .description_optional(resource.description)
        }
        Ok(ResolveResult::Ignored) => {
          let ident = format!("{}/{}", &data.context, request_without_match_resource);
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
          let ident = format!("{}/{request_without_match_resource}", &data.context);
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
      .calculate_module_rules(
        if let Some(match_resource_data) = match_resource_data.as_ref() {
          match_resource_data
        } else {
          &resource_data
        },
        data.dependency.category(),
      )
      .await?;

    let user_request = {
      let suffix = stringify_loaders_and_resource(&inline_loaders, &resource_data.resource);
      if let Some(ResourceData { resource, .. }) = match_resource_data.as_ref() {
        let mut resource = resource.to_owned();
        resource += "!=!";
        resource += &*suffix;
        resource
      } else {
        suffix.into_owned()
      }
    };

    // dbg!(&user_request);

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
      if match_resource_data.is_some() {
        all_loaders.extend(normal_loaders);
        all_loaders.extend(inline_loaders);
      } else {
        all_loaders.extend(inline_loaders);
        all_loaders.extend(normal_loaders);
      }
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
      side_effects: self.calculate_side_effects(&resolved_module_rules, &resource_data),
    };

    let resolved_parser_and_generator = self
      .plugin_driver
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
      match_resource_data,
      resource_data,
      resolved_resolve_options,
      loaders,
      self.context.options.clone(),
    );

    let module = if let Some(module) = self
      .plugin_driver
      .module(ModuleArgs {
        dependency_type: data.dependency.dependency_type().clone(),
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
    module_rules_matcher(
      &self.context.options.module.rules,
      resource_data,
      self.context.issuer.as_deref(),
      dependency,
      &mut rules,
    )
    .await?;
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

  fn calculate_side_effects(
    &self,
    module_rules: &[&ModuleRule],
    resource_data: &ResourceData,
  ) -> Option<bool> {
    let mut side_effect_res = None;
    // side_effects from module rule has higher priority
    module_rules.iter().for_each(|rule| {
      side_effect_res = rule.side_effects;
    });
    if side_effect_res.is_some() {
      return side_effect_res;
    }
    let resource_path = &resource_data.resource_path;
    let description = resource_data.resource_description.as_ref()?;
    let package_path = description.dir().as_ref();
    let side_effects = SideEffects::from_description(description)?;

    let relative_path = resource_path.relative(package_path);
    side_effect_res = Some(get_side_effects_from_package_json(
      side_effects,
      relative_path,
    ));

    side_effect_res
  }

  fn calculate_parser_and_generator_options(
    &self,
    module_rules: &[&ModuleRule],
  ) -> (Option<ParserOptions>, Option<GeneratorOptions>) {
    let mut resolved_parser = None;
    let mut resolved_generator = None;

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
    data: &mut ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let result = self
      .plugin_driver
      .factorize(
        FactorizeArgs {
          context: &data.context,
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

#[derive(Debug, Clone)]
pub struct NormalModuleFactoryContext {
  pub original_module_identifier: Option<ModuleIdentifier>,
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

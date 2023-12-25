use std::{
  path::Path,
  sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::{internal_error, Diagnosable, Diagnostic, Result};
use rspack_loader_runner::{get_scheme, Loader, Scheme};
use sugar_path::{AsPath, SugarPath};
use swc_core::common::Span;

use crate::{
  cache::Cache,
  diagnostics::EmptyDependency,
  module_rules_matcher, parse_resource, resolve, stringify_loaders_and_resource,
  tree_shaking::visitor::{get_side_effects_from_package_json, SideEffects},
  BoxLoader, CompilerContext, CompilerOptions, DependencyCategory, FactorizeArgs, FactoryMeta,
  FuncUseCtx, GeneratorOptions, MissingModule, ModuleExt, ModuleFactory, ModuleFactoryCreateData,
  ModuleFactoryResult, ModuleIdentifier, ModuleRule, ModuleRuleEnforce, ModuleRuleUse,
  ModuleRuleUseLoader, ModuleType, NormalModule, NormalModuleAfterResolveArgs,
  NormalModuleBeforeResolveArgs, NormalModuleCreateData, ParserOptions, RawModule, Resolve,
  ResolveArgs, ResolveError, ResolveOptionsWithDependencyType, ResolveResult, Resolver,
  ResolverFactory, ResourceData, ResourceParsedData, SharedPluginDriver,
};

#[derive(Debug)]
pub struct NormalModuleFactory {
  options: Arc<CompilerOptions>,
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
  diagnostics: Mutex<Vec<Diagnostic>>,
}

#[async_trait::async_trait]
impl ModuleFactory for NormalModuleFactory {
  async fn create(
    &self,
    mut data: ModuleFactoryCreateData,
  ) -> Result<(ModuleFactoryResult, Vec<Diagnostic>)> {
    let take_diagnostic = || {
      self
        .diagnostics
        .lock()
        .expect("should lock diagnostics")
        .drain(..)
        .collect::<Vec<_>>()
    };
    if let Ok(Some(before_resolve_data)) = self.before_resolve(&mut data).await {
      return Ok((before_resolve_data, take_diagnostic()));
    }
    let factory_result = self.factorize(&mut data).await?;
    if let Ok(Some(after_resolve_data)) = self.after_resolve(&data, &factory_result).await {
      return Ok((after_resolve_data, take_diagnostic()));
    }

    Ok((factory_result, take_diagnostic()))
  }
}

static MATCH_RESOURCE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("^([^!]+)!=!").expect("Failed to initialize `MATCH_RESOURCE_REGEX`"));

static MATCH_WEBPACK_EXT_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r#"\.webpack\[([^\]]+)\]$"#).expect("Failed to initialize `MATCH_WEBPACK_EXT_REGEX`")
});

impl NormalModuleFactory {
  pub fn new(
    options: Arc<CompilerOptions>,
    loader_resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      options,
      loader_resolver_factory,
      plugin_driver,
      cache,
      diagnostics: Default::default(),
    }
  }

  async fn before_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    let dependency = data
      .dependency
      .as_module_dependency_mut()
      .expect("should be module dependency");
    // allow javascript plugin to modify args
    let mut before_resolve_args = NormalModuleBeforeResolveArgs {
      request: dependency.request().to_string(),
      context: data.context.to_string(),
    };
    if let Ok(Some(false)) = self
      .plugin_driver
      .before_resolve(&mut before_resolve_args)
      .await
    {
      let request_without_match_resource = dependency.request();
      let ident = format!("{}/{request_without_match_resource}", &data.context);
      let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

      let missing_module = MissingModule::new(
        module_identifier,
        format!("{ident} (missing)"),
        format!("Failed to resolve {request_without_match_resource}"),
      )
      .boxed();
      return Ok(Some(ModuleFactoryResult::new(missing_module)));
    }

    data.context = before_resolve_args.context.into();
    dependency.set_request(before_resolve_args.request);
    Ok(None)
  }

  async fn after_resolve(
    &self,
    data: &ModuleFactoryCreateData,
    factory_result: &ModuleFactoryResult,
  ) -> Result<Option<ModuleFactoryResult>> {
    let dependency = data
      .dependency
      .as_module_dependency()
      .expect("should be module dependency");
    if let Ok(Some(false)) = self
      .plugin_driver
      .after_resolve(NormalModuleAfterResolveArgs {
        request: dependency.request(),
        context: data.context.as_ref(),
        file_dependencies: &factory_result.file_dependencies,
        context_dependencies: &factory_result.context_dependencies,
        missing_dependencies: &factory_result.missing_dependencies,
        factory_meta: &factory_result.factory_meta,
      })
      .await
    {
      let request_without_match_resource = dependency.request();
      let ident = format!("{}/{request_without_match_resource}", &data.context);
      let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

      let missing_module = MissingModule::new(
        module_identifier,
        format!("{ident} (missing)"),
        format!("Failed to resolve {request_without_match_resource}"),
      )
      .boxed();
      return Ok(Some(ModuleFactoryResult::new(missing_module)));
    }
    Ok(None)
  }

  fn get_loader_resolver(&self) -> Arc<Resolver> {
    self
      .loader_resolver_factory
      .get(ResolveOptionsWithDependencyType {
        resolve_options: None,
        resolve_to_context: false,
        dependency_category: DependencyCategory::CommonJS,
      })
  }

  pub async fn factorize_normal_module(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    let dependency = data
      .dependency
      .as_module_dependency()
      .expect("should be module dependency");
    let importer = data.issuer_identifier.as_ref();
    let mut request_without_match_resource = dependency.request();

    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();

    let scheme = get_scheme(request_without_match_resource);
    let context_scheme = get_scheme(data.context.as_ref());
    let context = data.context.as_path();
    let plugin_driver = &self.plugin_driver;
    let loader_resolver = self.get_loader_resolver();

    let mut match_resource_data: Option<ResourceData> = None;
    let mut match_module_type = None;
    let mut inline_loaders: Vec<ModuleRuleUseLoader> = vec![];
    let mut no_pre_auto_loaders = false;
    let mut no_auto_loaders = false;
    let mut no_pre_post_auto_loaders = false;

    // with scheme, windows absolute path is considered scheme by `url`
    let (resource_data, from_cache) = if scheme != Scheme::None
      && !Path::is_absolute(Path::new(request_without_match_resource))
    {
      // resource with scheme
      (
        plugin_driver
          .normal_module_factory_resolve_for_scheme(ResourceData::new(
            request_without_match_resource.to_string(),
            "".into(),
          ))
          .await?,
        false,
      )
    }
    // TODO: resource within scheme, call resolveInScheme hook
    else {
      {
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
              .nth(whole_matched.chars().count())
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

        if first_char.is_none() {
          let span = dependency.source_span().unwrap_or_default();
          return Err(EmptyDependency::new(span).into());
        }

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
            None => request_without_match_resource,
          };
          s.split('!')
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
        };

        request_without_match_resource = raw_elements
          .pop()
          .ok_or_else(|| internal_error!("Invalid request: {request_without_match_resource}"))?;

        inline_loaders.extend(raw_elements.into_iter().map(|r| ModuleRuleUseLoader {
          loader: r.to_owned(),
          options: None,
        }));
      }
      let optional = dependency.get_optional();

      let resolve_args = ResolveArgs {
        importer,
        context: if context_scheme != Scheme::None {
          self.options.context.clone()
        } else {
          data.context.clone()
        },
        specifier: request_without_match_resource,
        dependency_type: dependency.dependency_type(),
        dependency_category: dependency.category(),
        span: dependency.span(),
        // take the options is safe here, because it
        // is not used in after_resolve hooks
        resolve_options: data.resolve_options.take(),
        resolve_to_context: false,
        optional,
        file_dependencies: &mut file_dependencies,
        missing_dependencies: &mut missing_dependencies,
      };

      // default resolve
      let (resource_data, from_cache) = match self
        .cache
        .resolve_module_occasion
        .use_cache(resolve_args, |args| resolve(args, plugin_driver))
        .await
      {
        Ok(result) => result,
        Err(err) => (Err(err), false),
      };
      match resource_data {
        Ok(ResolveResult::Resource(resource)) => {
          let uri = resource.full_path().display().to_string();
          (
            ResourceData::new(uri, resource.path)
              .query_optional(resource.query)
              .fragment_optional(resource.fragment)
              .description_optional(resource.description_data),
            from_cache,
          )
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

          return Ok(Some(
            ModuleFactoryResult::new(raw_module).from_cache(from_cache),
          ));
        }
        Err(ResolveError(runtime_error, internal_error)) => {
          let ident = format!("{}/{request_without_match_resource}", &data.context);
          let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));
          let mut file_dependencies = Default::default();
          let mut missing_dependencies = Default::default();
          self.add_diagnostic(internal_error.into());

          let mut from_cache_result = from_cache;
          if !data
            .resolve_options
            .as_ref()
            .and_then(|x| x.fully_specified)
            .unwrap_or(false)
          {
            let new_args = ResolveArgs {
              importer,
              context: if context_scheme != Scheme::None {
                self.options.context.clone()
              } else {
                data.context.clone()
              },
              specifier: request_without_match_resource,
              dependency_type: dependency.dependency_type(),
              dependency_category: dependency.category(),
              resolve_options: data.resolve_options.take(),
              span: dependency.span(),
              resolve_to_context: false,
              optional,
              missing_dependencies: &mut missing_dependencies,
              file_dependencies: &mut file_dependencies,
            };
            let (resource_data, from_cache) = match self
              .cache
              .resolve_module_occasion
              .use_cache(new_args, |args| resolve(args, plugin_driver))
              .await
            {
              Ok(result) => result,
              Err(err) => (Err(err), false),
            };
            from_cache_result = from_cache;
            if let Ok(ResolveResult::Resource(resource)) = resource_data {
              // TODO: Here windows resolver will return normalized path.
              // eg. D:\a\rspack\rspack\packages\rspack\tests\fixtures\errors\resolve-fail-esm\answer.js
              if let Some(_extension) = resource.path.extension() {
                // let resource = format!(
                //   "{request_without_match_resource}.{}",
                //   extension.to_string_lossy()
                // );
                // diagnostics[0].add_notes(vec![format!("Did you mean '{resource}'?
                // BREAKING CHANGE: The request '{request_without_match_resource}' failed to resolve only because it was resolved as fully specified
                // (probably because the origin is strict EcmaScript Module, e. g. a module with javascript mimetype, a '*.mjs' file, or a '*.js' file where the package.json contains '\"type\": \"module\"').
                // The extension in the request is mandatory for it to be fully specified.
                // Add the extension to the request.")]);
              }
            }
          }
          let missing_module = MissingModule::new(
            module_identifier,
            format!("{ident} (missing)"),
            runtime_error,
          )
          .boxed();
          return Ok(Some(
            ModuleFactoryResult::new(missing_module).from_cache(from_cache_result),
          ));
        }
      }
    };

    let resolved_module_rules = if let Some(match_resource_data) = &mut match_resource_data
      && let Some(captures) = MATCH_WEBPACK_EXT_REGEX.captures(&match_resource_data.resource)
      && let Some(module_type) = captures.get(1)
    {
      match_module_type = Some(module_type.as_str().into());
      match_resource_data.resource = match_resource_data
        .resource
        .strip_suffix(&format!(".webpack[{}]", module_type.as_str()))
        .expect("should success")
        .to_owned();

      vec![]
    } else {
      //TODO: with contextScheme
      self
        .calculate_module_rules(
          if let Some(match_resource_data) = match_resource_data.as_ref() {
            match_resource_data
          } else {
            &resource_data
          },
          data.dependency.category(),
          data.issuer.as_deref(),
        )
        .await?
    };

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
    let contains_inline = !inline_loaders.is_empty();

    let loaders: Vec<BoxLoader> = {
      let mut pre_loaders: Vec<ModuleRuleUseLoader> = vec![];
      let mut post_loaders: Vec<ModuleRuleUseLoader> = vec![];
      let mut normal_loaders: Vec<ModuleRuleUseLoader> = vec![];

      for rule in &resolved_module_rules {
        match &rule.r#use {
          ModuleRuleUse::Array(array_use) => match rule.enforce {
            ModuleRuleEnforce::Pre => {
              if !no_pre_auto_loaders && !no_pre_post_auto_loaders {
                pre_loaders.extend_from_slice(array_use);
              }
            }
            ModuleRuleEnforce::Normal => {
              if !no_auto_loaders && !no_pre_auto_loaders {
                normal_loaders.extend_from_slice(array_use);
              }
            }
            ModuleRuleEnforce::Post => {
              if !no_pre_post_auto_loaders {
                post_loaders.extend_from_slice(array_use);
              }
            }
          },
          ModuleRuleUse::Func(func_use) => {
            let context = FuncUseCtx {
              resource: Some(resource_data.resource.clone()),
              real_resource: Some(user_request.clone()),
              issuer: data.issuer.clone(),
              resource_query: resource_data.resource_query.clone(),
            };
            let loaders = func_use(context).await?;

            normal_loaders.extend(loaders);
          }
        }
      }

      let mut all_loaders = Vec::with_capacity(
        pre_loaders.len() + post_loaders.len() + normal_loaders.len() + inline_loaders.len(),
      );

      for l in post_loaders {
        all_loaders.push(
          resolve_each(
            plugin_driver,
            &self.options,
            self.options.context.as_ref(),
            &loader_resolver,
            &l.loader,
            l.options.as_deref(),
          )
          .await?,
        )
      }

      let mut resolved_inline_loaders = vec![];
      let mut resolved_normal_loaders = vec![];

      for l in inline_loaders {
        resolved_inline_loaders.push(
          resolve_each(
            plugin_driver,
            &self.options,
            context,
            &loader_resolver,
            &l.loader,
            l.options.as_deref(),
          )
          .await?,
        )
      }

      for l in normal_loaders {
        resolved_normal_loaders.push(
          resolve_each(
            plugin_driver,
            &self.options,
            self.options.context.as_ref(),
            &loader_resolver,
            &l.loader,
            l.options.as_deref(),
          )
          .await?,
        )
      }

      if match_resource_data.is_some() {
        all_loaders.extend(resolved_normal_loaders);
        all_loaders.extend(resolved_inline_loaders);
      } else {
        all_loaders.extend(resolved_inline_loaders);
        all_loaders.extend(resolved_normal_loaders);
      }

      for l in pre_loaders {
        all_loaders.push(
          resolve_each(
            plugin_driver,
            &self.options,
            self.options.context.as_ref(),
            &loader_resolver,
            &l.loader,
            l.options.as_deref(),
          )
          .await?,
        )
      }

      async fn resolve_each(
        plugin_driver: &SharedPluginDriver,
        compiler_options: &CompilerOptions,
        context: &Path,
        loader_resolver: &Resolver,
        loader_request: &str,
        loader_options: Option<&str>,
      ) -> Result<Arc<dyn Loader<CompilerContext>>> {
        plugin_driver
          .resolve_loader(
            compiler_options,
            context,
            loader_resolver,
            loader_request,
            loader_options,
          )
          .await?
          .ok_or_else(|| internal_error!("Unable to resolve loader {}", loader_request))
      }

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
      self.calculate_module_type(match_module_type, &resolved_module_rules);
    let resolved_resolve_options = self.calculate_resolve_options(&resolved_module_rules);
    let (resolved_parser_options, resolved_generator_options) =
      self.calculate_parser_and_generator_options(&resolved_module_rules);
    let factory_meta = FactoryMeta {
      side_effect_free: self
        .calculate_side_effects(&resolved_module_rules, &resource_data)
        .map(|side_effects| !side_effects),
    };

    let resolved_parser_and_generator = self
      .plugin_driver
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        let mut e = format!("Unexpected `ModuleType` found: {resolved_module_type}. ");

        match resolved_module_type {
          ModuleType::Css => e.push_str(
            "Setting `'css'` as the `Rule.type` is only possible with `experiments.css` set to `true`",
          ),
          ModuleType::Ts | ModuleType::Tsx |
          ModuleType::Jsx | ModuleType::JsxDynamic | ModuleType::JsxEsm => e.push_str("`Rule.type` that are not supported by webpack is deprecated. See: https://github.com/web-infra-dev/rspack/discussions/4070"),
          _ => (),
        }

        internal_error!(e)
      })?();

    let create_data = NormalModuleCreateData {
      dependency_type: data.dependency.dependency_type().clone(),
      resolve_data_request: dependency.request(),
      resource_resolve_data: resource_data.clone(),
      context: data.context.clone(),
      normal_module_factory: self,
    };
    let module = if let Some(module) = self
      .plugin_driver
      .normal_module_factory_create_module(&create_data)
      .await?
    {
      module
    } else {
      let normal_module = NormalModule::new(
        request,
        user_request,
        dependency.request().to_owned(),
        resolved_module_type,
        resolved_parser_and_generator,
        resolved_parser_options,
        resolved_generator_options,
        match_resource_data,
        resource_data,
        resolved_resolve_options,
        loaders,
        self.options.clone(),
        contains_inline,
      );
      Box::new(normal_module)
    };

    let module = self
      .plugin_driver
      .normal_module_factory_module(module, &create_data)
      .await?;

    Ok(Some(
      ModuleFactoryResult::new(module)
        .file_dependency(file_dependency)
        .file_dependencies(file_dependencies)
        .missing_dependencies(missing_dependencies)
        .factory_meta(factory_meta)
        .from_cache(from_cache),
    ))
  }

  async fn calculate_module_rules<'a>(
    &'a self,
    resource_data: &ResourceData,
    dependency: &DependencyCategory,
    issuer: Option<&'a str>,
  ) -> Result<Vec<&'a ModuleRule>> {
    let mut rules = Vec::new();
    module_rules_matcher(
      &self.options.module.rules,
      resource_data,
      issuer,
      dependency,
      &mut rules,
    )
    .await?;
    Ok(rules)
  }

  fn calculate_resolve_options(&self, module_rules: &[&ModuleRule]) -> Option<Box<Resolve>> {
    let mut resolved = None;
    module_rules.iter().for_each(|rule| {
      if let Some(resolve) = rule.resolve.as_ref() {
        resolved = Some(Box::new(resolve.to_owned()));
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
      if rule.side_effects.is_some() {
        side_effect_res = rule.side_effects;
      }
    });
    if side_effect_res.is_some() {
      return side_effect_res;
    }
    let resource_path = &resource_data.resource_path;
    let description = resource_data.resource_description.as_ref()?;
    let package_path = description.path();
    let side_effects = SideEffects::from_description(description.json())?;

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

  fn calculate_module_type(
    &self,
    matched_module_type: Option<ModuleType>,
    module_rules: &[&ModuleRule],
  ) -> ModuleType {
    let mut resolved_module_type = matched_module_type.unwrap_or(ModuleType::Js);

    module_rules.iter().for_each(|module_rule| {
      if let Some(module_type) = module_rule.r#type {
        resolved_module_type = module_type;
      };
    });

    resolved_module_type
  }

  async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dependency = data
      .dependency
      .as_module_dependency()
      .expect("should be module dependency");
    let result = self
      .plugin_driver
      .factorize(FactorizeArgs {
        context: &data.context,
        dependency,
        plugin_driver: &self.plugin_driver,
        module_factory: self,
      })
      .await?;

    if let Some(result) = result {
      return Ok(result);
    }

    if let Some(result) = self.factorize_normal_module(data).await? {
      return Ok(result);
    }

    Err(internal_error!(
      "Failed to factorize module, neither hook nor factorize method returns"
    ))
  }
}

impl Diagnosable for NormalModuleFactory {
  fn add_diagnostic(&self, diagnostic: Diagnostic) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .push(diagnostic);
  }

  fn add_diagnostics(&self, mut diagnostics: Vec<Diagnostic>) {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .append(&mut diagnostics);
  }

  fn clone_diagnostics(&self) -> Vec<Diagnostic> {
    self
      .diagnostics
      .lock()
      .expect("should be able to lock diagnostics")
      .iter()
      .cloned()
      .collect()
  }
}

/// Using `u32` instead of `usize` to reduce memory usage,
/// `u32` is 4 bytes on 64bit machine, comare to `usize` which is 8 bytes.
/// Rspan aka `Rspack span`, just avoiding conflict with span in other crate
/// ## Warning
/// RSpan is zero based, `Span` of `swc` is 1 based. see https://swc-css.netlify.app/?code=eJzLzC3ILypRSFRIK8rPVVAvSS0u0csqVgcAZaoIKg
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
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

#[test]
fn match_webpack_ext() {
  assert!(MATCH_WEBPACK_EXT_REGEX.is_match("foo.webpack[type/javascript]"));
  let cap = MATCH_WEBPACK_EXT_REGEX
    .captures("foo.webpack[type/javascript]")
    .unwrap();

  assert_eq!(cap.get(1).unwrap().as_str(), "type/javascript");
}

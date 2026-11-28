use std::{
  borrow::Cow,
  sync::{Arc, LazyLock, UniqueArc},
};

use memchr::{memchr, memmem, memrchr};
use rspack_error::{Result, error};
use rspack_hook::define_hook;
use rspack_loader_runner::{Loader, LoaderFilter, LoaderRunnerOptions, Scheme, get_scheme};
use sugar_path::SugarPath;

use crate::{
  BoxLoader, BoxModule, CompilerOptions, Context, Dependency, DependencyCategory, DependencyType,
  FactoryMeta, FuncUseCtx, GeneratorOptions, MatchContext, ModuleExt, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, ModuleRuleEffectIndices,
  ModuleRuleEnforce, ModuleRuleUse, ModuleRuleUseLoader, ModuleType, NormalModule,
  ParserAndGenerator, ParserOptions, RawModule, ResolveArgs, ResolveOptionsWithDependencyType,
  ResolveResult, Resolver, ResolverFactory, ResourceData, ResourceParsedData, RunnerContext,
  RuntimeGlobals, SharedPluginDriver, diagnostics::EmptyDependency, module_rules_matcher,
  parse_resource, resolve, stringify_loaders_and_resource,
};

define_hook!(NormalModuleFactoryBeforeResolve: SeriesBail(data: &mut ModuleFactoryCreateData) -> bool,tracing=false);
define_hook!(NormalModuleFactoryFactorize: SeriesBail(data: &mut ModuleFactoryCreateData) -> BoxModule,tracing=false);
define_hook!(NormalModuleFactoryResolve: SeriesBail(data: &mut ModuleFactoryCreateData) -> NormalModuleFactoryResolveResult,tracing=false);
define_hook!(NormalModuleFactoryResolveForScheme: SeriesBail(data: &mut ModuleFactoryCreateData, resource_data: &mut ResourceData, for_name: &Scheme) -> bool,tracing=false);
define_hook!(NormalModuleFactoryResolveInScheme: SeriesBail(data: &mut ModuleFactoryCreateData, resource_data: &mut ResourceData, for_name: &Scheme) -> bool,tracing=false);
define_hook!(NormalModuleFactoryAfterResolve: SeriesBail(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData) -> bool,tracing=false);
define_hook!(NormalModuleFactoryCreateModule: SeriesBail(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData) -> BoxModule,tracing=false);
define_hook!(NormalModuleFactoryModule: Series(data: &mut ModuleFactoryCreateData, create_data: &NormalModuleCreateData, module: &mut BoxModule),tracing=false);
define_hook!(NormalModuleFactoryParser: Series(module_type: &ModuleType, parser: &mut Box<dyn ParserAndGenerator>, parser_options: Option<&ParserOptions>),tracing=false);
define_hook!(NormalModuleFactoryResolveLoader: SeriesBail(context: &Context, resolver: &Resolver, l: &ModuleRuleUseLoader, data: &mut ModuleFactoryCreateData) -> BoxLoader,tracing=false);
define_hook!(NormalModuleFactoryAfterFactorize: Series(data: &mut ModuleFactoryCreateData, module: &mut BoxModule),tracing=false);

pub enum NormalModuleFactoryResolveResult {
  Module(BoxModule),
}

#[derive(Debug, Default)]
pub struct NormalModuleFactoryHooks {
  pub before_resolve: NormalModuleFactoryBeforeResolveHook,
  pub factorize: NormalModuleFactoryFactorizeHook,
  pub resolve: NormalModuleFactoryResolveHook,
  pub resolve_for_scheme: NormalModuleFactoryResolveForSchemeHook,
  pub resolve_in_scheme: NormalModuleFactoryResolveInSchemeHook,
  pub after_resolve: NormalModuleFactoryAfterResolveHook,
  pub create_module: NormalModuleFactoryCreateModuleHook,
  pub module: NormalModuleFactoryModuleHook,
  pub parser: NormalModuleFactoryParserHook,
  /// Webpack resolves loaders in `NormalModuleFactory`,
  /// Rspack resolves it when normalizing configuration.
  /// So this hook is used to resolve inline loader (inline loader requests).
  // should move to ResolverFactory?
  pub resolve_loader: NormalModuleFactoryResolveLoaderHook,
  pub after_factorize: NormalModuleFactoryAfterFactorizeHook,
}

#[derive(Debug)]
pub struct NormalModuleFactory {
  options: Arc<CompilerOptions>,
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
}

#[async_trait::async_trait]
impl ModuleFactory for NormalModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Some(before_resolve_data) = self.before_resolve(data).await? {
      return Ok(before_resolve_data);
    }
    let mut factory_result = self.factorize(data).await?;

    if let Some(module) = &mut factory_result.module {
      self
        .plugin_driver
        .normal_module_factory_hooks
        .after_factorize
        .call(data, module)
        .await?;
    }

    Ok(factory_result)
  }
}

const HYPHEN: char = '-';
const EXCLAMATION: char = '!';
const DOT: char = '.';
const SLASH: char = '/';
const QUESTION_MARK: char = '?';

impl NormalModuleFactory {
  pub fn new(
    options: Arc<CompilerOptions>,
    loader_resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
  ) -> Self {
    Self {
      options,
      loader_resolver_factory,
      plugin_driver,
    }
  }

  async fn before_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    if let Some(false) = self
      .plugin_driver
      .normal_module_factory_hooks
      .before_resolve
      .call(data)
      .await?
    {
      // ignored
      // See https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/NormalModuleFactory.js#L798
      return Ok(Some(ModuleFactoryResult::default()));
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

  async fn resolve_normal_module(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    let dependency = data.dependencies[0]
      .as_module_dependency()
      .expect("should be module dependency");
    let dependency_type = *dependency.dependency_type();
    let dependency_category = *dependency.category();
    let dependency_range = dependency.range();
    let dependency_optional = dependency.get_optional();
    let dependency_phase = dependency.get_phase();

    let importer = data.issuer_identifier;
    let raw_request = data.request.clone();

    let context = data.context.clone();

    let plugin_driver = &self.plugin_driver;
    let loader_resolver = self.get_loader_resolver();

    let mut match_resource_data = None;
    let mut match_module_type = None;
    let mut inline_loaders = vec![];
    let mut no_pre_auto_loaders = false;
    let mut no_auto_loaders = false;
    let mut no_pre_post_auto_loaders = false;

    let mut scheme = get_scheme(&data.request);
    let context_scheme = get_scheme(data.context.as_ref());
    let mut unresolved_resource = data.request.as_str();
    if scheme.is_none() {
      let mut request_without_match_resource = data.request.as_str();
      request_without_match_resource = {
        if let Some((resource, full_matched)) = match_resource(request_without_match_resource) {
          let match_resource = {
            let mut chars = resource.chars();
            let first_char = chars.next();
            let second_char = chars.next();

            if matches!(first_char, Some(DOT))
              && (matches!(second_char, Some(SLASH))
                || (matches!(second_char, Some(DOT)) && matches!(chars.next(), Some(SLASH))))
            {
              // if matchResources startsWith ../ or ./
              data
                .context
                .as_path()
                .join(resource)
                .as_std_path()
                .absolutize()
                .to_string_lossy()
                .into_owned()
            } else {
              resource.to_owned()
            }
          };

          let ResourceParsedData {
            path,
            query,
            fragment,
          } = parse_resource(&match_resource).expect("Should parse resource");
          match_resource_data = Some(ResourceData::new_with_path(
            match_resource,
            path,
            query,
            fragment,
          ));

          // e.g. ./index.js!=!
          let whole_matched = full_matched;

          match request_without_match_resource
            .char_indices()
            .nth(whole_matched.chars().count())
          {
            Some((pos, _)) => &request_without_match_resource[pos..],
            None => {
              unreachable!("Invalid dependency: {:?}", &data.dependencies[0])
            }
          }
        } else {
          request_without_match_resource
        }
      };

      scheme = get_scheme(request_without_match_resource);
      if scheme.is_none() && context_scheme.is_none() {
        let mut request = request_without_match_resource.chars();
        let first_char = request.next();
        let second_char = request.next();

        if first_char.is_none() {
          return Err(EmptyDependency::new(dependency_range).into());
        }

        // See: https://webpack.js.org/concepts/loaders/#inline
        no_pre_auto_loaders =
          matches!(first_char, Some(HYPHEN)) && matches!(second_char, Some(EXCLAMATION));
        no_auto_loaders = no_pre_auto_loaders || matches!(first_char, Some(EXCLAMATION));
        no_pre_post_auto_loaders =
          matches!(first_char, Some(EXCLAMATION)) && matches!(second_char, Some(EXCLAMATION));

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
          split_element(s)
        };

        unresolved_resource = raw_elements
          .pop()
          .ok_or_else(|| error!("Invalid request: {request_without_match_resource}"))?;

        inline_loaders.extend(raw_elements.into_iter().map(|r| {
          let resource = parse_resource(r);
          let ident = resource.as_ref().and_then(|r| {
            r.query
              .as_ref()
              .and_then(|q| q.starts_with("??").then(|| &q[2..]))
          });
          ModuleRuleUseLoader {
            loader: r.to_owned(),
            options: ident.and_then(|ident| {
              data
                .build_context
                .compiler_options
                .__references
                .get(ident)
                .map(|object| object.to_string())
            }),
            cache: false,
            options_cache_key: String::new(),
            filter: None,
          }
        }));
        scheme = get_scheme(unresolved_resource);
      } else {
        unresolved_resource = request_without_match_resource;
      }
    }

    let resource = unresolved_resource.to_owned();
    let resource_data = if !scheme.is_none() {
      // resource with scheme
      let mut resource_data = UniqueArc::new(ResourceData::new_with_resource(resource));
      plugin_driver
        .normal_module_factory_hooks
        .resolve_for_scheme
        .call(data, &mut resource_data, &scheme)
        .await?;
      resource_data
    } else if !context_scheme.is_none()
      // resource within scheme
      && let Some(resource_data) = {
        let mut resource_data = UniqueArc::new(ResourceData::new_with_resource(resource.clone()));
        let handled = plugin_driver
          .normal_module_factory_hooks
          .resolve_in_scheme
          .call(data, &mut resource_data, &context_scheme)
          .await?
          .unwrap_or_default();
        handled.then_some(resource_data)
      }
    {
      resource_data
    } else {
      // resource without scheme and without path
      if resource.is_empty() || resource.starts_with(QUESTION_MARK) {
        UniqueArc::new(ResourceData::new_with_resource(resource))
      } else {
        // resource without scheme and with path
        let resolve_args = ResolveArgs {
          importer: importer.as_ref(),
          issuer: data.issuer.as_deref(),
          context: if context_scheme != Scheme::None {
            self.options.context.clone()
          } else {
            data.context.clone()
          },
          specifier: &resource,
          dependency_type: &dependency_type,
          dependency_category: &dependency_category,
          span: dependency_range,
          resolve_options: data.resolve_options.clone(),
          resolve_to_context: false,
          optional: dependency_optional,
        };

        let (resource_data, resolve_dependencies) = resolve(resolve_args, plugin_driver).await;
        data
          .file_dependencies
          .extend(resolve_dependencies.file_dependencies);
        data
          .missing_dependencies
          .extend(resolve_dependencies.missing_dependencies);

        match resource_data {
          Ok(ResolveResult::Resource(resource)) => UniqueArc::new(resource.into()),
          Ok(ResolveResult::Ignored) => {
            let ident = format!("{}/{}", data.context, resource);

            let raw_module = if matches!(
              dependency_type,
              DependencyType::CssUrl | DependencyType::NewUrl
            ) {
              // use RawModule instead of RawDataUrlModule
              // Keep ignored assets separate from ignored JS modules in the graph and cache.
              RawModule::new(
                r#"/* (ignored-asset) */
module.exports = "data:,";
"#
                .to_owned(),
                ModuleIdentifier::from(format!("ignored-asset|{ident}")),
                format!("{} (ignored-asset)", data.request),
                RuntimeGlobals::MODULE,
              )
              .boxed()
            } else {
              RawModule::new(
                "/* (ignored) */".to_owned(),
                ModuleIdentifier::from(format!("ignored|{ident}")),
                format!("{} (ignored)", data.request),
                Default::default(),
              )
              .boxed()
            };

            raw_module.set_factory_meta(FactoryMeta {
              side_effect_free: Some(true),
            });

            return Ok(Some(ModuleFactoryResult::new_with_module(raw_module)));
          }
          Err(err) => return Err(err),
        }
      }
    };

    let effect_indices = if let Some(match_resource_data) = &mut match_resource_data
      && let Some((module, module_type)) = match_ext(match_resource_data.resource())
    {
      match_module_type = Some(module_type.into());
      match_resource_data.set_resource(module.into());

      ModuleRuleEffectIndices::new()
    } else {
      //TODO: with contextScheme
      self
        .match_rule_effects(
          if let Some(match_resource_data) = match_resource_data.as_ref() {
            match_resource_data
          } else {
            &resource_data
          },
          data.dependencies[0].as_ref(),
          data.issuer.as_deref(),
          data.issuer_layer.as_deref(),
        )
        .await?
    };
    let mut resolved_inline_loaders = vec![];
    for l in inline_loaders {
      resolved_inline_loaders
        .push(resolve_each(plugin_driver, &context, &loader_resolver, &l, data).await?)
    }

    let user_request = {
      let suffix =
        stringify_loaders_and_resource(&resolved_inline_loaders, resource_data.resource());
      if let Some(match_resource_data) = &match_resource_data {
        let mut resource = match_resource_data.resource().to_owned();
        resource += "!=!";
        resource += &*suffix;
        resource
      } else {
        suffix.into_owned()
      }
    };

    let resolved_loaders: Vec<ResolvedLoader> = {
      let mut pre_loaders: Vec<ModuleRuleUseLoader> = vec![];
      let mut post_loaders: Vec<ModuleRuleUseLoader> = vec![];
      let mut normal_loaders: Vec<ModuleRuleUseLoader> = vec![];

      for &idx in &effect_indices {
        let effect = &self.options.module.effects[idx];
        let rule_use = match &effect.r#use {
          ModuleRuleUse::Array(array_use) => Cow::Borrowed(array_use),
          ModuleRuleUse::Func(func_use) => {
            let resource_data_for_rules = match_resource_data.as_ref().unwrap_or(&resource_data);
            let context = FuncUseCtx {
              // align with webpack https://github.com/webpack/webpack/blob/899f06934391baede59da3dcd35b5ef51c675dbe/lib/NormalModuleFactory.js#L576
              resource: resource_data_for_rules.path().map(|x| x.to_string()),
              resource_query: resource_data_for_rules.query().map(|q| q.to_owned()),
              resource_fragment: resource_data_for_rules.fragment().map(|f| f.to_owned()),
              real_resource: resource_data.path().map(|p| p.to_string()),
              issuer: data.issuer.clone(),
              issuer_layer: data.issuer_layer.clone(),
            };
            Cow::Owned(func_use(context).await?)
          }
        };

        match effect.enforce {
          ModuleRuleEnforce::Pre => {
            if !no_pre_auto_loaders && !no_pre_post_auto_loaders {
              pre_loaders.extend_from_slice(&rule_use);
            }
          }
          ModuleRuleEnforce::Normal => {
            if !no_auto_loaders && !no_pre_auto_loaders {
              normal_loaders.extend_from_slice(&rule_use);
            }
          }
          ModuleRuleEnforce::Post => {
            if !no_pre_post_auto_loaders {
              post_loaders.extend_from_slice(&rule_use);
            }
          }
        }
      }

      let mut all_loaders = Vec::with_capacity(
        pre_loaders.len()
          + post_loaders.len()
          + normal_loaders.len()
          + resolved_inline_loaders.len(),
      );

      for l in post_loaders {
        all_loaders.push(
          resolve_each_with_options(plugin_driver, &self.options, &loader_resolver, &l, data)
            .await?,
        )
      }

      let mut resolved_normal_loaders = vec![];
      for l in normal_loaders {
        resolved_normal_loaders.push(
          resolve_each_with_options(plugin_driver, &self.options, &loader_resolver, &l, data)
            .await?,
        )
      }

      if match_resource_data.is_some() {
        all_loaders.extend(resolved_normal_loaders);
        all_loaders.extend(
          resolved_inline_loaders
            .into_iter()
            .map(ResolvedLoader::uncached),
        );
      } else {
        all_loaders.extend(
          resolved_inline_loaders
            .into_iter()
            .map(ResolvedLoader::uncached),
        );
        all_loaders.extend(resolved_normal_loaders);
      }

      for l in pre_loaders {
        all_loaders.push(
          resolve_each_with_options(plugin_driver, &self.options, &loader_resolver, &l, data)
            .await?,
        )
      }

      all_loaders
    };

    let request = if !resolved_loaders.is_empty() {
      let s = resolved_loaders
        .iter()
        .map(|i| i.loader.identifier().as_str())
        .collect::<Vec<_>>()
        .join("!");
      format!("{s}!{}", resource_data.resource())
    } else {
      resource_data.resource().to_owned()
    };
    let mut has_cached_loader = false;
    let mut has_filter = false;
    for resolved in &resolved_loaders {
      has_cached_loader |= resolved.options.cache;
      has_filter |= resolved.filter.is_some();
    }
    let loader_filters = has_filter.then(|| {
      resolved_loaders
        .iter()
        .map(|resolved| resolved.filter.clone())
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });
    let (loaders, loader_options) = if has_cached_loader {
      let (loaders, loader_options) = resolved_loaders
        .into_iter()
        .map(|resolved| (resolved.loader, resolved.options))
        .unzip();
      (loaders, Some(loader_options))
    } else {
      (
        resolved_loaders
          .into_iter()
          .map(|resolved| resolved.loader)
          .collect(),
        None,
      )
    };

    let mut rule_module_type = None;
    let mut resolved_module_layer = data.issuer_layer.as_ref();
    let mut resolved_side_effects = None;
    let mut resolved_extract_source_map = None;
    for &idx in &effect_indices {
      let effect = &self.options.module.effects[idx];
      rule_module_type = effect.r#type.or(rule_module_type);
      resolved_module_layer = effect.layer.as_ref().or(resolved_module_layer);
      resolved_side_effects = effect.side_effects.or(resolved_side_effects);
      resolved_extract_source_map = effect.extract_source_map.or(resolved_extract_source_map);
    }
    // Like webpack, `!!` also disables Rule.type, but preserves an explicit matchResource type.
    let resolved_module_type = if no_pre_post_auto_loaders {
      match_module_type
    } else {
      rule_module_type.or(match_module_type)
    }
    .unwrap_or(ModuleType::JsAuto);
    let resolved_module_layer = resolved_module_layer.cloned();
    let rule_options = self
      .options
      .module
      .get_rule_options(&effect_indices, resolved_module_type);
    let mut resolved_parser_and_generator = self
      .plugin_driver
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        error!(
          "No parser registered for '{}'",
          resolved_module_type.as_str()
        )
      })?(
      rule_options.parser.as_deref(),
      rule_options.generator.as_deref(),
    );
    self
      .plugin_driver
      .normal_module_factory_hooks
      .parser
      .call(
        &resolved_module_type,
        &mut resolved_parser_and_generator,
        rule_options.parser.as_deref(),
      )
      .await?;

    let mut create_data = {
      let mut create_data = NormalModuleCreateData {
        raw_request,
        request,
        user_request,
        match_resource: match_resource_data
          .as_ref()
          .map(|d| d.resource().to_owned()),
        side_effects: resolved_side_effects,
        context: resource_data.context().map(|c| c.to_owned()),
        parser_options: rule_options.parser.clone(),
        generator_options: rule_options.generator.clone(),
        resource_resolve_data: UniqueArc::into_arc(resource_data),
      };
      if let Some(plugin_result) = self
        .plugin_driver
        .normal_module_factory_hooks
        .after_resolve
        .call(data, &mut create_data)
        .await?
        && !plugin_result
      {
        // ignored
        // See https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/NormalModuleFactory.js#L301
        return Ok(Some(ModuleFactoryResult::default()));
      }

      create_data
    };

    let mut module = if let Some(module) = self
      .plugin_driver
      .normal_module_factory_hooks
      .create_module
      .call(data, &mut create_data)
      .await?
    {
      module
    } else {
      NormalModule::new(
        create_data.request.clone(),
        create_data.user_request.clone(),
        create_data.raw_request.clone(),
        resolved_module_type,
        resolved_module_layer,
        resolved_parser_and_generator,
        effect_indices,
        match_resource_data,
        Arc::clone(&create_data.resource_resolve_data),
        loaders,
        loader_options,
        loader_filters,
        create_data.context.clone().map(|x| x.into()),
        resolved_extract_source_map,
        dependency_phase,
      )
      .boxed()
    };

    self
      .plugin_driver
      .normal_module_factory_hooks
      .module
      .call(data, &create_data, &mut module)
      .await?;

    Ok(Some(ModuleFactoryResult::new_with_module(module)))
  }

  async fn match_rule_effects(
    &self,
    resource_data: &ResourceData,
    dependency: &dyn Dependency,
    issuer: Option<&str>,
    issuer_layer: Option<&str>,
  ) -> Result<ModuleRuleEffectIndices> {
    let mut effect_indices = ModuleRuleEffectIndices::new();
    let match_ctx = MatchContext {
      resource_data,
      issuer,
      issuer_layer,
      dependency: *dependency.category(),
      phase: dependency.get_phase(),
      attributes: dependency.get_attributes(),
    };
    module_rules_matcher(&self.options.module.rules, &match_ctx, &mut effect_indices).await?;
    Ok(effect_indices)
  }

  async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Some(result) = self
      .plugin_driver
      .normal_module_factory_hooks
      .factorize
      .call(data)
      .await?
    {
      return Ok(ModuleFactoryResult::new_with_module(result));
    }

    if let Some(result) = self
      .plugin_driver
      .normal_module_factory_hooks
      .resolve
      .call(data)
      .await?
    {
      let NormalModuleFactoryResolveResult::Module(result) = result;
      return Ok(ModuleFactoryResult::new_with_module(result));
    }

    if let Some(result) = self.resolve_normal_module(data).await? {
      return Ok(result);
    }

    Err(error!(
      "Failed to factorize module, neither hook nor factorize method returns"
    ))
  }
}

async fn resolve_each(
  plugin_driver: &SharedPluginDriver,
  context: &Context,
  loader_resolver: &Resolver,
  l: &ModuleRuleUseLoader,
  data: &mut ModuleFactoryCreateData,
) -> Result<Arc<dyn Loader<RunnerContext>>> {
  plugin_driver
    .normal_module_factory_hooks
    .resolve_loader
    .call(context, loader_resolver, l, data)
    .await?
    .ok_or_else(|| error!("Can't resolve loader '{}' in '{context}'", l.loader))
}

struct ResolvedLoader {
  loader: BoxLoader,
  options: LoaderRunnerOptions,
  filter: Option<Arc<LoaderFilter>>,
}

impl ResolvedLoader {
  fn uncached(loader: BoxLoader) -> Self {
    Self {
      loader,
      options: LoaderRunnerOptions::default(),
      filter: None,
    }
  }
}

async fn resolve_each_with_options(
  plugin_driver: &SharedPluginDriver,
  options: &CompilerOptions,
  loader_resolver: &Resolver,
  loader: &ModuleRuleUseLoader,
  data: &mut ModuleFactoryCreateData,
) -> Result<ResolvedLoader> {
  let uncached_loader;
  let loader = if options.experiments.new_cache.loader {
    loader
  } else {
    uncached_loader = ModuleRuleUseLoader {
      cache: false,
      ..loader.clone()
    };
    &uncached_loader
  };
  let resolved = resolve_each(
    plugin_driver,
    &options.context,
    loader_resolver,
    loader,
    data,
  )
  .await?;
  if !loader.cache {
    return Ok(ResolvedLoader {
      loader: resolved,
      options: LoaderRunnerOptions::default(),
      filter: loader.filter.clone(),
    });
  }
  let loader_name = parse_resource(&loader.loader).map_or_else(
    || loader.loader.clone(),
    |resource| resource.path.to_string(),
  );
  let loader_version = resolved
    .cache_version()
    .unwrap_or(rspack_workspace::rspack_pkg_version!())
    .to_owned();
  Ok(ResolvedLoader {
    loader: resolved,
    options: LoaderRunnerOptions {
      cache: true,
      loader_name,
      options_cache_key: loader.options_cache_key.clone(),
      loader_version,
    },
    filter: loader.filter.clone(),
  })
}

#[derive(Debug)]
pub struct NormalModuleCreateData {
  pub parser_options: Option<Arc<ParserOptions>>,
  pub generator_options: Option<Arc<GeneratorOptions>>,
  pub raw_request: String,
  pub request: String,
  pub user_request: String,
  resource_resolve_data: Arc<ResourceData>,
  pub match_resource: Option<String>,
  pub side_effects: Option<bool>,
  pub context: Option<String>,
}

impl NormalModuleCreateData {
  pub fn resource_resolve_data(&self) -> &ResourceData {
    &self.resource_resolve_data
  }

  pub fn update_resource_data(&mut self, new_resource: String) {
    Arc::get_mut(&mut self.resource_resolve_data)
      .expect("shared resource resolve data cannot be mutated after module creation")
      .update_resource_data(new_resource);
  }
}

fn split_element(mut input: &str) -> Vec<&str> {
  let mut elements = Vec::new();
  while let Some(end) = memchr(b'!', input.as_bytes()) {
    elements.push(&input[..end]);
    // Consecutive separators form one split point.
    let rest = &input[end + 1..];
    let skip = rest
      .as_bytes()
      .iter()
      .take_while(|&&byte| byte == b'!')
      .count();
    input = &rest[skip..];
  }
  elements.push(input);
  elements
}

fn match_resource(input: &str) -> Option<(&str, &str)> {
  let end = memchr(b'!', input.as_bytes())?;
  if end == 0 {
    return None;
  }
  let rest = input[end..].strip_prefix("!=!")?;
  Some((&input[..end], &input[..input.len() - rest.len()]))
}

fn match_ext(input: &str) -> Option<(&str, &str)> {
  static MARKERS: LazyLock<[memmem::Finder<'static>; 2]> = LazyLock::new(|| {
    [
      memmem::Finder::new(b".rspack["),
      memmem::Finder::new(b".webpack["),
    ]
  });

  let input = input.strip_suffix(']')?;
  let bytes = input.as_bytes();
  // A suffix can only start after the previous closing bracket.
  let suffix_start = memrchr(b']', bytes).map_or(0, |index| index + 1);
  for marker in MARKERS.iter() {
    if let Some(start) = marker.find(&bytes[suffix_start..]) {
      let start = suffix_start + start;
      let module_type = &input[start + marker.needle().len()..];
      if !module_type.is_empty() {
        return Some((&input[..start], module_type));
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_split_element() {
    assert_eq!(split_element("a!a"), vec!["a", "a"]);
    assert_eq!(split_element("a!!a"), vec!["a", "a"]);
    assert_eq!(split_element("!!a!!a!!"), vec!["", "a", "a", ""]);
  }

  #[test]
  fn test_match_ext() {
    assert!(match_ext("foo.webpack[type/javascript]").is_some());
    let cap = match_ext("foo.webpack[type/javascript]").unwrap();

    assert_eq!(cap, ("foo", "type/javascript"));

    assert_eq!(
      match_ext("foo.css.webpack[javascript/auto]"),
      Some(("foo.css", "javascript/auto"))
    );

    // Test .rspack support
    assert!(match_ext("foo.rspack[type/javascript]").is_some());
    let cap = match_ext("foo.rspack[type/javascript]").unwrap();

    assert_eq!(cap, ("foo", "type/javascript"));

    assert_eq!(
      match_ext("foo.css.rspack[javascript/auto]"),
      Some(("foo.css", "javascript/auto"))
    );
  }
}

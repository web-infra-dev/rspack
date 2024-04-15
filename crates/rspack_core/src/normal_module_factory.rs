use std::{borrow::Cow, path::Path, sync::Arc};

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_error::{error, Result};
use rspack_hook::define_hook;
use rspack_loader_runner::{get_scheme, Loader, Scheme};
use rspack_util::MergeFrom;
use sugar_path::SugarPath;
use swc_core::common::Span;

use crate::{
  cache::Cache,
  diagnostics::EmptyDependency,
  module_rules_matcher, parse_resource, resolve, stringify_loaders_and_resource,
  tree_shaking::visitor::{get_side_effects_from_package_json, SideEffects},
  BoxLoader, BoxModule, CompilerContext, CompilerOptions, Context, DependencyCategory, FactoryMeta,
  FuncUseCtx, GeneratorOptions, ModuleExt, ModuleFactory, ModuleFactoryCreateData,
  ModuleFactoryResult, ModuleIdentifier, ModuleRule, ModuleRuleEnforce, ModuleRuleUse,
  ModuleRuleUseLoader, ModuleType, NormalModule, ParserOptions, RawModule, Resolve, ResolveArgs,
  ResolveOptionsWithDependencyType, ResolveResult, Resolver, ResolverFactory, ResourceData,
  ResourceParsedData, SharedPluginDriver,
};

define_hook!(NormalModuleFactoryBeforeResolve: AsyncSeriesBail(data: &mut ModuleFactoryCreateData) -> bool);
define_hook!(NormalModuleFactoryFactorize: AsyncSeriesBail(data: &mut ModuleFactoryCreateData) -> BoxModule);
define_hook!(NormalModuleFactoryResolveForScheme: AsyncSeriesBail(data: &mut ModuleFactoryCreateData, resource_data: &mut ResourceData) -> bool);
define_hook!(NormalModuleFactoryAfterResolve: AsyncSeriesBail(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData) -> bool);
define_hook!(NormalModuleFactoryCreateModule: AsyncSeriesBail(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData) -> BoxModule);
define_hook!(NormalModuleFactoryModule: AsyncSeries(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData, module: &mut BoxModule));
define_hook!(NormalModuleFactoryResolveLoader: AsyncSeriesBail(context: &Context, resolver: &Resolver, l: &ModuleRuleUseLoader) -> BoxLoader);

#[derive(Debug, Default)]
pub struct NormalModuleFactoryHooks {
  pub before_resolve: NormalModuleFactoryBeforeResolveHook,
  pub factorize: NormalModuleFactoryFactorizeHook,
  pub resolve_for_scheme: NormalModuleFactoryResolveForSchemeHook,
  pub after_resolve: NormalModuleFactoryAfterResolveHook,
  pub create_module: NormalModuleFactoryCreateModuleHook,
  pub module: NormalModuleFactoryModuleHook,
  /// Webpack resolves loaders in `NormalModuleFactory`,
  /// Rspack resolves it when normalizing configuration.
  /// So this hook is used to resolve inline loader (inline loader requests).
  // should move to ResolverFactory?
  pub resolve_loader: NormalModuleFactoryResolveLoaderHook,
}

#[derive(Debug)]
pub struct NormalModuleFactory {
  options: Arc<CompilerOptions>,
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
}

#[async_trait::async_trait]
impl ModuleFactory for NormalModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Some(before_resolve_data) = self.before_resolve(data).await? {
      return Ok(before_resolve_data);
    }
    let factory_result = self.factorize(data).await?;

    Ok(factory_result)
  }
}

static MATCH_RESOURCE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new("^([^!]+)!=!").expect("Failed to initialize `MATCH_RESOURCE_REGEX`"));

static MATCH_WEBPACK_EXT_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r#"\.webpack\[([^\]]+)\]$"#).expect("Failed to initialize `MATCH_WEBPACK_EXT_REGEX`")
});

static ELEMENT_SPLIT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"!+").expect("Failed to initialize `ELEMENT_SPLIT_REGEX`"));

const HYPHEN: char = '-';
const EXCLAMATION: char = '!';

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

  pub async fn factorize_normal_module(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    let dependency = data
      .dependency
      .as_module_dependency()
      .expect("should be module dependency");
    let importer = data.issuer_identifier.as_ref();
    let raw_request = dependency.request().to_owned();
    let mut request_without_match_resource = dependency.request();

    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();

    let scheme = get_scheme(request_without_match_resource);
    let context_scheme = get_scheme(data.context.as_ref());
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
      let mut resource_data =
        ResourceData::new(request_without_match_resource.to_string(), "".into());
      // resource with scheme
      plugin_driver
        .normal_module_factory_hooks
        .resolve_for_scheme
        .call(data, &mut resource_data)
        .await?;
      (resource_data, false)
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
              match_resource = data
                .context
                .as_path()
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
          ELEMENT_SPLIT_REGEX.split(s).collect::<Vec<_>>()
        };

        request_without_match_resource = raw_elements
          .pop()
          .ok_or_else(|| error!("Invalid request: {request_without_match_resource}"))?;

        inline_loaders.extend(raw_elements.into_iter().map(|r| ModuleRuleUseLoader {
          loader: r.to_owned(),
          options: None,
        }));
      }

      if request_without_match_resource.is_empty()
        || request_without_match_resource.starts_with('?')
      {
        let ResourceParsedData {
          path,
          query,
          fragment,
        } = parse_resource(request_without_match_resource).expect("Should parse resource");
        let resource_data = ResourceData::new(request_without_match_resource.to_string(), path)
          .query_optional(query)
          .fragment_optional(fragment);
        (resource_data, false)
      } else {
        let optional = dependency.get_optional();

        let resolve_args = ResolveArgs {
          importer,
          issuer: data.issuer.as_deref(),
          context: if context_scheme != Scheme::None {
            self.options.context.clone()
          } else {
            data.context.clone()
          },
          specifier: request_without_match_resource,
          dependency_type: dependency.dependency_type(),
          dependency_category: dependency.category(),
          span: dependency.source_span(),
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
                .query(resource.query)
                .fragment(resource.fragment)
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
              ModuleFactoryResult::new_with_module(raw_module).from_cache(from_cache),
            ));
          }
          Err(err) => {
            data.add_file_dependencies(file_dependencies);
            data.add_missing_dependencies(missing_dependencies);
            return Err(err);
          }
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
        let rule_use = match &rule.r#use {
          ModuleRuleUse::Array(array_use) => Cow::Borrowed(array_use),
          ModuleRuleUse::Func(func_use) => {
            let context = FuncUseCtx {
              resource: Some(resource_data.resource.clone()),
              real_resource: Some(user_request.clone()),
              issuer: data.issuer.clone(),
              resource_query: resource_data.resource_query.clone(),
            };
            Cow::Owned(func_use(context).await?)
          }
        };

        match rule.enforce {
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
        pre_loaders.len() + post_loaders.len() + normal_loaders.len() + inline_loaders.len(),
      );

      for l in post_loaders {
        all_loaders
          .push(resolve_each(plugin_driver, &self.options.context, &loader_resolver, &l).await?)
      }

      let mut resolved_inline_loaders = vec![];
      let mut resolved_normal_loaders = vec![];

      for l in inline_loaders {
        resolved_inline_loaders
          .push(resolve_each(plugin_driver, &data.context, &loader_resolver, &l).await?)
      }

      for l in normal_loaders {
        resolved_normal_loaders
          .push(resolve_each(plugin_driver, &self.options.context, &loader_resolver, &l).await?)
      }

      if match_resource_data.is_some() {
        all_loaders.extend(resolved_normal_loaders);
        all_loaders.extend(resolved_inline_loaders);
      } else {
        all_loaders.extend(resolved_inline_loaders);
        all_loaders.extend(resolved_normal_loaders);
      }

      for l in pre_loaders {
        all_loaders
          .push(resolve_each(plugin_driver, &self.options.context, &loader_resolver, &l).await?)
      }

      async fn resolve_each(
        plugin_driver: &SharedPluginDriver,
        context: &Context,
        loader_resolver: &Resolver,
        l: &ModuleRuleUseLoader,
      ) -> Result<Arc<dyn Loader<CompilerContext>>> {
        plugin_driver
          .normal_module_factory_hooks
          .resolve_loader
          .call(context, loader_resolver, l)
          .await?
          .ok_or_else(|| error!("Unable to resolve loader {}", l.loader))
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
    let (resolved_parser_options, resolved_generator_options) = self
      .merge_global_parser_and_generator_options(
        &resolved_module_type,
        resolved_parser_options,
        resolved_generator_options,
      );
    let resolved_side_effects = self.calculate_side_effects(&resolved_module_rules);
    let resolved_parser_and_generator = self
      .plugin_driver
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        error!(
          "No parser registered for '{}'",
          resolved_module_type.as_str()
        )
      })?(
      resolved_parser_options.as_ref(),
      resolved_generator_options.as_ref(),
    );

    let resource_path = resource_data.resource_path.clone();
    let resource_description = resource_data.resource_description.clone();

    let mut create_data = {
      let mut create_data = NormalModuleCreateData {
        raw_request,
        request,
        user_request,
        resource_resolve_data: resource_data,
        match_resource: match_resource_data.as_ref().map(|d| d.resource.clone()),
        side_effects: resolved_side_effects,
      };
      if let Some(plugin_result) = self
        .plugin_driver
        .normal_module_factory_hooks
        .after_resolve
        .call(data, &mut create_data)
        .await?
      {
        if !plugin_result {
          // ignored
          // See https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/NormalModuleFactory.js#L301
          return Ok(Some(ModuleFactoryResult::default()));
        }
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
        resolved_parser_and_generator,
        resolved_parser_options,
        resolved_generator_options,
        match_resource_data,
        create_data.resource_resolve_data.clone(),
        resolved_resolve_options,
        loaders,
        contains_inline,
      )
      .boxed()
    };

    self
      .plugin_driver
      .normal_module_factory_hooks
      .module
      .call(data, &mut create_data, &mut module)
      .await?;

    data.add_file_dependencies(file_dependencies);
    data.add_file_dependency(file_dependency);
    data.add_missing_dependencies(missing_dependencies);

    // Compat for old tree shaking
    if !self.options.is_new_tree_shaking() {
      if resolved_side_effects.is_some() {
        module.set_factory_meta(FactoryMeta {
          side_effect_free: None,
          side_effect_free_old: resolved_side_effects.map(|has_side_effects| !has_side_effects),
        })
      } else if let Some(description) = resource_description.as_ref()
        && let Some(side_effects) = SideEffects::from_description(description.json())
      {
        let package_path = description.path();
        let relative_path = resource_path.relative(package_path);
        module.set_factory_meta(FactoryMeta {
          side_effect_free: None,
          side_effect_free_old: Some(!get_side_effects_from_package_json(
            side_effects,
            relative_path,
          )),
        })
      }
    }

    Ok(Some(
      ModuleFactoryResult::new_with_module(module).from_cache(from_cache),
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

  fn calculate_side_effects(&self, module_rules: &[&ModuleRule]) -> Option<bool> {
    let mut side_effect_res = None;
    // side_effects from module rule has higher priority
    module_rules.iter().for_each(|rule| {
      if rule.side_effects.is_some() {
        side_effect_res = rule.side_effects;
      }
    });
    side_effect_res
  }

  fn calculate_parser_and_generator_options(
    &self,
    module_rules: &[&ModuleRule],
  ) -> (Option<ParserOptions>, Option<GeneratorOptions>) {
    let mut resolved_parser = None;
    let mut resolved_generator = None;

    for rule in module_rules {
      resolved_parser = resolved_parser.merge_from(&rule.parser);
      resolved_generator = resolved_generator.merge_from(&rule.generator);
    }

    (resolved_parser, resolved_generator)
  }

  fn merge_global_parser_and_generator_options(
    &self,
    module_type: &ModuleType,
    parser: Option<ParserOptions>,
    generator: Option<GeneratorOptions>,
  ) -> (Option<ParserOptions>, Option<GeneratorOptions>) {
    let global_parser = self
      .options
      .module
      .parser
      .as_ref()
      .and_then(|p| p.get(module_type))
      .cloned();
    let global_generator = self
      .options
      .module
      .generator
      .as_ref()
      .and_then(|g| g.get(module_type))
      .cloned();
    let parser = rspack_util::merge_from_optional_with(
      global_parser,
      parser.as_ref(),
      |global, local| match (&global, local) {
        (ParserOptions::Asset(_), ParserOptions::Asset(_))
        | (ParserOptions::Css(_), ParserOptions::Css(_))
        | (ParserOptions::CssAuto(_), ParserOptions::CssAuto(_))
        | (ParserOptions::CssModule(_), ParserOptions::CssModule(_))
        | (ParserOptions::Javascript(_), ParserOptions::Javascript(_)) => global.merge_from(local),
        _ => global,
      },
    );
    let generator = rspack_util::merge_from_optional_with(
      global_generator,
      generator.as_ref(),
      |global, local| match (&global, local) {
        (GeneratorOptions::Asset(_), GeneratorOptions::Asset(_))
        | (GeneratorOptions::AssetInline(_), GeneratorOptions::AssetInline(_))
        | (GeneratorOptions::AssetResource(_), GeneratorOptions::AssetResource(_))
        | (GeneratorOptions::Css(_), GeneratorOptions::Css(_))
        | (GeneratorOptions::CssAuto(_), GeneratorOptions::CssAuto(_))
        | (GeneratorOptions::CssModule(_), GeneratorOptions::CssModule(_)) => {
          global.merge_from(local)
        }
        _ => global,
      },
    );
    (parser, generator)
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
    let result = self
      .plugin_driver
      .normal_module_factory_hooks
      .factorize
      .call(data)
      .await?;

    if let Some(result) = result {
      return Ok(ModuleFactoryResult::new_with_module(result));
    }

    if let Some(result) = self.factorize_normal_module(data).await? {
      return Ok(result);
    }

    Err(error!(
      "Failed to factorize module, neither hook nor factorize method returns"
    ))
  }
}

/// Using `u32` instead of `usize` to reduce memory usage,
/// `u32` is 4 bytes on 64bit machine, comparing to `usize` which is 8 bytes.
/// ## Warning
/// [ErrorSpan] start from zero, and `Span` of `swc` start from one. see https://swc-css.netlify.app/?code=eJzLzC3ILypRSFRIK8rPVVAvSS0u0csqVgcAZaoIKg
#[derive(
  Debug,
  Hash,
  PartialEq,
  Eq,
  Clone,
  Copy,
  Default,
  PartialOrd,
  Ord,
  rkyv::Archive,
  rkyv::Serialize,
  rkyv::Deserialize,
)]
#[archive(check_bytes)]
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

#[derive(Debug)]
pub struct NormalModuleCreateData {
  pub raw_request: String,
  pub request: String,
  pub user_request: String,
  pub resource_resolve_data: ResourceData,
  pub match_resource: Option<String>,
  pub side_effects: Option<bool>,
}

#[test]
fn match_webpack_ext() {
  assert!(MATCH_WEBPACK_EXT_REGEX.is_match("foo.webpack[type/javascript]"));
  let cap = MATCH_WEBPACK_EXT_REGEX
    .captures("foo.webpack[type/javascript]")
    .unwrap();

  assert_eq!(cap.get(1).unwrap().as_str(), "type/javascript");
}

use std::{borrow::Cow, collections::HashMap, ops::Deref, sync::Arc};

use rspack_error::{Result, error};
use rspack_hook::define_hook;
use rspack_loader_runner::{Loader, Scheme, get_scheme};
use rspack_paths::ArcResolverPathSet;
use rspack_util::{MergeFrom, fx_hash::FxDashMap};
use sugar_path::SugarPath;
use winnow::prelude::*;

use crate::{
  AssetInlineGeneratorOptions, AssetResourceGeneratorOptions, BoxLoader, BoxModule,
  CompilerOptions, Context, CssModuleGeneratorOptions, CssModuleParserOptions, Dependency,
  DependencyCategory, DependencyType, FactoryMeta, FuncUseCtx, GeneratorOptions, ModuleExt,
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, ModuleLayer,
  ModuleRuleEffect, ModuleRuleEnforce, ModuleRuleUse, ModuleRuleUseLoader, ModuleType,
  NormalModule, ParserAndGenerator, ParserOptions, ParserOptionsMap, RawModule, Resolve,
  ResolveArgs, ResolveOptionsWithDependencyType, ResolveResult, ResolvedModuleOptions,
  ResolvedModuleOptionsCacheKey, Resolver, ResolverFactory, ResourceData, ResourceParsedData,
  RunnerContext, RuntimeGlobals, SharedPluginDriver, diagnostics::EmptyDependency,
  module_rules_matcher, parse_resource, resolve, stringify_loaders_and_resource,
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
define_hook!(NormalModuleFactoryResolveLoader: SeriesBail(context: &Context, resolver: &Resolver, l: &ModuleRuleUseLoader) -> BoxLoader,tracing=false);
define_hook!(NormalModuleFactoryAfterFactorize: Series(data: &mut ModuleFactoryCreateData, module: &mut BoxModule),tracing=false);

pub enum NormalModuleFactoryResolveResult {
  Module(BoxModule),
  Ignored,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum NormalModuleCreateDataResource {
  Owned(ResourceData),
  Shared(Arc<ResourceData>),
}

impl NormalModuleCreateDataResource {
  fn into_shared(self) -> Arc<ResourceData> {
    match self {
      Self::Owned(resource_data) => Arc::new(resource_data),
      Self::Shared(resource_data) => resource_data,
    }
  }

  fn take_shared(&mut self) -> Arc<ResourceData> {
    let shared = std::mem::take(self).into_shared();
    *self = Self::Shared(shared.clone());
    shared
  }

  pub fn update_resource_data(&mut self, new_resource: String) {
    match self {
      Self::Owned(resource_data) => resource_data.update_resource_data(new_resource),
      Self::Shared(_) => {
        panic!("shared resource resolve data cannot be mutated after module creation")
      }
    }
  }
}

impl Default for NormalModuleCreateDataResource {
  fn default() -> Self {
    Self::Owned(ResourceData::default())
  }
}

impl Deref for NormalModuleCreateDataResource {
  type Target = ResourceData;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Owned(resource_data) => resource_data,
      Self::Shared(resource_data) => resource_data,
    }
  }
}

impl AsRef<ResourceData> for NormalModuleCreateDataResource {
  fn as_ref(&self) -> &ResourceData {
    self
  }
}

fn create_global_parser_options_cache(
  parser_options: Option<&ParserOptionsMap>,
) -> HashMap<String, ParserOptions> {
  let mut cache = HashMap::new();
  let Some(parser_options) = parser_options else {
    return cache;
  };

  for (key, options) in parser_options.iter() {
    cache.insert(key.clone(), options.clone());
  }

  for module_type in [
    ModuleType::JsAuto,
    ModuleType::JsDynamic,
    ModuleType::JsEsm,
    ModuleType::CssAuto,
    ModuleType::CssGlobal,
    ModuleType::CssModule,
  ] {
    if let Some(options) = resolve_global_parser_options(parser_options, &module_type) {
      cache.insert(module_type.as_str().to_owned(), options);
    }
  }

  cache
}

fn create_global_generator_options_cache(
  generator_options: Option<&crate::GeneratorOptionsMap>,
) -> HashMap<String, GeneratorOptions> {
  let mut cache = HashMap::new();
  let Some(generator_options) = generator_options else {
    return cache;
  };

  for (key, options) in generator_options.iter() {
    cache.insert(key.clone(), options.clone());
  }

  for module_type in [
    ModuleType::AssetInline,
    ModuleType::AssetResource,
    ModuleType::CssAuto,
    ModuleType::CssGlobal,
    ModuleType::CssModule,
    ModuleType::Json,
  ] {
    if let Some(options) = resolve_global_generator_options(generator_options, &module_type) {
      cache.insert(module_type.as_str().to_owned(), options);
    }
  }

  cache
}

fn resolve_global_generator_options(
  generator_options: &crate::GeneratorOptionsMap,
  module_type: &ModuleType,
) -> Option<GeneratorOptions> {
  let options = generator_options.get(module_type.as_str());
  match module_type {
    ModuleType::AssetInline | ModuleType::AssetResource => rspack_util::merge_from_optional_with(
      generator_options.get("asset").cloned(),
      options,
      |asset_options, options| match (asset_options, options) {
        (GeneratorOptions::Asset(a), GeneratorOptions::AssetInline(b)) => {
          GeneratorOptions::AssetInline(Into::<AssetInlineGeneratorOptions>::into(a).merge_from(b))
        }
        (GeneratorOptions::Asset(a), GeneratorOptions::AssetResource(b)) => {
          GeneratorOptions::AssetResource(
            Into::<AssetResourceGeneratorOptions>::into(a).merge_from(b),
          )
        }
        _ => unreachable!(),
      },
    ),
    ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule => {
      rspack_util::merge_from_optional_with(
        generator_options.get("css").cloned(),
        options,
        |css_options, options| match (css_options, options) {
          (GeneratorOptions::Css(a), GeneratorOptions::CssModule(b)) => {
            GeneratorOptions::CssModule(Into::<CssModuleGeneratorOptions>::into(&a).merge_from(b))
          }
          _ => unreachable!(),
        },
      )
    }
    ModuleType::Json => rspack_util::merge_from_optional_with(
      generator_options.get("json").cloned(),
      options,
      |json_options, options| match (json_options, options) {
        (GeneratorOptions::Json(a), GeneratorOptions::Json(b)) => {
          GeneratorOptions::Json(a.merge_from(b))
        }
        _ => unreachable!(),
      },
    ),
    _ => options.cloned(),
  }
}

fn resolve_global_parser_options(
  parser_options: &ParserOptionsMap,
  module_type: &ModuleType,
) -> Option<ParserOptions> {
  let options = parser_options.get(module_type.as_str());
  match module_type {
    ModuleType::JsAuto | ModuleType::JsDynamic | ModuleType::JsEsm => {
      // Merge `module.parser.["javascript/xxx"]` with `module.parser.["javascript"]` first.
      rspack_util::merge_from_optional_with(
        parser_options.get("javascript").cloned(),
        options,
        |javascript_options, options| match (javascript_options, options) {
          (
            ParserOptions::Javascript(a),
            ParserOptions::JavascriptAuto(b)
            | ParserOptions::JavascriptDynamic(b)
            | ParserOptions::JavascriptEsm(b),
          ) => ParserOptions::Javascript(a.merge_from(b)),
          _ => unreachable!(),
        },
      )
    }
    ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule => {
      rspack_util::merge_from_optional_with(
        parser_options.get("css").cloned(),
        options,
        |css_options, options| match (css_options, options) {
          (ParserOptions::Css(a), ParserOptions::CssModule(b)) => {
            ParserOptions::CssModule(Into::<CssModuleParserOptions>::into(&a).merge_from(b))
          }
          _ => unreachable!(),
        },
      )
    }
    _ => options.cloned(),
  }
}

fn merge_parser_options_with_local(
  global_parser: Option<&ParserOptions>,
  local_parser: Option<&ParserOptions>,
) -> Option<ParserOptions> {
  match (global_parser, local_parser) {
    (None, None) => None,
    (None, Some(local)) => Some(local.clone()),
    (Some(global), None) => Some(global.clone()),
    (Some(global), Some(local)) => Some(match (global, local) {
      (ParserOptions::Asset(a), ParserOptions::Asset(b)) => {
        ParserOptions::Asset(a.clone().merge_from(b))
      }
      (ParserOptions::Css(a), ParserOptions::Css(b)) => ParserOptions::Css(a.clone().merge_from(b)),
      (ParserOptions::CssModule(a), ParserOptions::CssModule(b)) => {
        ParserOptions::CssModule(a.clone().merge_from(b))
      }
      (
        ParserOptions::Javascript(a),
        ParserOptions::JavascriptAuto(b)
        | ParserOptions::JavascriptDynamic(b)
        | ParserOptions::JavascriptEsm(b),
      ) => ParserOptions::Javascript(a.clone().merge_from(b)),
      (ParserOptions::Json(a), ParserOptions::Json(b)) => {
        ParserOptions::Json(a.clone().merge_from(b))
      }
      (global, _) => global.clone(),
    }),
  }
}

fn merge_generator_options_with_local(
  global_generator: Option<&GeneratorOptions>,
  local_generator: Option<&GeneratorOptions>,
) -> Option<GeneratorOptions> {
  match (global_generator, local_generator) {
    (None, None) => None,
    (None, Some(local)) => Some(local.clone()),
    (Some(global), None) => Some(global.clone()),
    (Some(global), Some(local)) => Some(match (global, local) {
      (GeneratorOptions::Asset(_), GeneratorOptions::Asset(_))
      | (GeneratorOptions::AssetInline(_), GeneratorOptions::AssetInline(_))
      | (GeneratorOptions::AssetResource(_), GeneratorOptions::AssetResource(_))
      | (GeneratorOptions::Css(_), GeneratorOptions::Css(_))
      | (GeneratorOptions::CssModule(_), GeneratorOptions::CssModule(_))
      | (GeneratorOptions::Json(_), GeneratorOptions::Json(_)) => global.clone().merge_from(local),
      (global, _) => global.clone(),
    }),
  }
}

fn normalize_css_parser_options(
  module_type: &ModuleType,
  parser: Option<ParserOptions>,
) -> Option<ParserOptions> {
  if !matches!(
    module_type,
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule
  ) {
    return parser;
  }

  match parser.as_ref() {
    Some(ParserOptions::Css(options)) => Some(ParserOptions::CssModule(
      CssModuleParserOptions::from(options),
    )),
    _ => parser,
  }
}

fn normalize_css_generator_options(
  module_type: &ModuleType,
  generator: Option<GeneratorOptions>,
) -> Option<GeneratorOptions> {
  if !matches!(
    module_type,
    ModuleType::Css | ModuleType::CssAuto | ModuleType::CssGlobal | ModuleType::CssModule
  ) {
    return generator;
  }

  match generator.as_ref() {
    Some(GeneratorOptions::Css(options)) => Some(GeneratorOptions::CssModule(
      CssModuleGeneratorOptions::from(options),
    )),
    _ => generator,
  }
}

fn resolve_module_options(
  options_cache: &FxDashMap<ResolvedModuleOptionsCacheKey, Arc<ResolvedModuleOptions>>,
  module_rules: &[&ModuleRuleEffect],
  module_type: &ModuleType,
  global_parser: Option<&ParserOptions>,
  global_generator: Option<&GeneratorOptions>,
) -> Arc<ResolvedModuleOptions> {
  let cache_key = ResolvedModuleOptionsCacheKey::new(module_rules, *module_type);
  if let Some(options) = options_cache.get(&cache_key) {
    return Arc::clone(&options);
  }

  match options_cache.entry(cache_key.clone()) {
    dashmap::mapref::entry::Entry::Occupied(entry) => Arc::clone(entry.get()),
    dashmap::mapref::entry::Entry::Vacant(entry) => {
      let mut local_parser = None;
      let mut local_generator = None;

      for rule in module_rules {
        local_parser = local_parser.merge_from(&rule.parser);
        local_generator = local_generator.merge_from(&rule.generator);
      }

      let parser = merge_parser_options_with_local(global_parser, local_parser.as_ref());
      let generator =
        merge_generator_options_with_local(global_generator, local_generator.as_ref());
      let options = Arc::new(ResolvedModuleOptions::new(
        cache_key,
        normalize_css_parser_options(module_type, parser),
        normalize_css_generator_options(module_type, generator),
      ));
      entry.insert(Arc::clone(&options));
      options
    }
  }
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
  global_parser_options: HashMap<String, ParserOptions>,
  global_generator_options: HashMap<String, GeneratorOptions>,
  resolved_module_options: FxDashMap<ResolvedModuleOptionsCacheKey, Arc<ResolvedModuleOptions>>,
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    AssetGeneratorOptions, AssetParserDataUrl, AssetParserDataUrlOptions, AssetParserOptions,
    CssGeneratorOptions, CssParserOptions, JavascriptParserOptions,
  };

  #[test]
  fn reuse_global_parser_options_when_local_options_are_empty() {
    let global = ParserOptions::Javascript(JavascriptParserOptions {
      jsx: Some(true),
      ..Default::default()
    });

    let resolved = merge_parser_options_with_local(Some(&global), None)
      .expect("global parser options should be reused");

    assert_eq!(
      resolved
        .get_javascript()
        .expect("javascript parser options should be resolved")
        .jsx,
      Some(true)
    );
  }

  #[test]
  fn merge_local_parser_options_into_global_parser_options() {
    let global = ParserOptions::Javascript(JavascriptParserOptions {
      jsx: Some(true),
      require_dynamic: Some(true),
      ..Default::default()
    });
    let local = ParserOptions::JavascriptAuto(JavascriptParserOptions {
      jsx: Some(false),
      ..Default::default()
    });

    let resolved = merge_parser_options_with_local(Some(&global), Some(&local))
      .expect("merged parser options should exist");

    let options = resolved
      .get_javascript()
      .expect("javascript parser options should be resolved");
    assert_eq!(options.jsx, Some(false));
    assert_eq!(options.require_dynamic, Some(true));
  }

  fn asset_rule_effect(max_size: f64, emit: bool) -> ModuleRuleEffect {
    ModuleRuleEffect {
      parser: Some(ParserOptions::Asset(AssetParserOptions {
        data_url_condition: Some(AssetParserDataUrl::Options(AssetParserDataUrlOptions {
          max_size: Some(max_size),
        })),
      })),
      generator: Some(GeneratorOptions::Asset(AssetGeneratorOptions {
        emit: Some(emit),
        ..Default::default()
      })),
      ..Default::default()
    }
  }

  #[test]
  fn lazily_reuse_parser_and_generator_options_for_rule_ids() {
    let mut module_options = crate::ModuleOptions {
      rules: vec![crate::ModuleRule {
        effect: asset_rule_effect(200.0, false),
        ..Default::default()
      }],
      ..Default::default()
    };
    module_options
      .assign_rule_ids()
      .expect("should assign rule ids");
    let rule = &module_options.rules[0].effect;
    let options_cache = FxDashMap::default();
    let module_rules = [rule];
    let global_parser = ParserOptions::Asset(AssetParserOptions {
      data_url_condition: Some(AssetParserDataUrl::Options(AssetParserDataUrlOptions {
        max_size: Some(100.0),
      })),
    });
    let global_generator = GeneratorOptions::Asset(AssetGeneratorOptions {
      emit: Some(true),
      ..Default::default()
    });

    let first = resolve_module_options(
      &options_cache,
      &module_rules,
      &ModuleType::Asset,
      Some(&global_parser),
      Some(&global_generator),
    );
    let second = resolve_module_options(
      &options_cache,
      &module_rules,
      &ModuleType::Asset,
      Some(&global_parser),
      Some(&global_generator),
    );

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(options_cache.len(), 1);
    assert_eq!(first.cache_key().rule_ids.as_slice(), &[rule.id]);
    assert_eq!(first.cache_key().module_type, ModuleType::Asset);

    let parser_options = first
      .parser_options()
      .and_then(ParserOptions::get_asset)
      .expect("asset parser options should exist");
    let max_size = match parser_options.data_url_condition.as_ref() {
      Some(AssetParserDataUrl::Options(options)) => options.max_size,
      _ => None,
    };
    assert_eq!(max_size, Some(200.0));

    let generator_options = first
      .generator_options()
      .and_then(GeneratorOptions::get_asset)
      .expect("asset generator options should exist");
    assert_eq!(generator_options.emit, Some(false));
  }

  #[test]
  fn reuse_global_generator_options_when_local_options_are_empty() {
    let options_cache = FxDashMap::default();
    let module_rules: [&ModuleRuleEffect; 0] = [];
    let global_generator = GeneratorOptions::Asset(AssetGeneratorOptions {
      emit: Some(true),
      ..Default::default()
    });

    let options = resolve_module_options(
      &options_cache,
      &module_rules,
      &ModuleType::Asset,
      None,
      Some(&global_generator),
    );

    assert_eq!(
      options
        .generator_options()
        .and_then(GeneratorOptions::get_asset)
        .expect("asset generator options should exist")
        .emit,
      Some(true)
    );
  }

  #[test]
  fn normalize_css_options_for_parser_and_generator_builder() {
    let options_cache = FxDashMap::default();
    let module_rules: [&ModuleRuleEffect; 0] = [];
    let options = resolve_module_options(
      &options_cache,
      &module_rules,
      &ModuleType::Css,
      Some(&ParserOptions::Css(CssParserOptions {
        named_exports: Some(true),
        url: None,
        r#import: None,
        resolve_import: None,
        animation: None,
        custom_idents: None,
        dashed_idents: None,
      })),
      Some(&GeneratorOptions::Css(CssGeneratorOptions {
        exports_only: Some(false),
        es_module: Some(true),
      })),
    );

    let parser_options = options
      .parser_options()
      .and_then(ParserOptions::get_css_module)
      .expect("css parser options should be normalized to CssModule");
    let generator_options = options
      .generator_options()
      .and_then(GeneratorOptions::get_css_module)
      .expect("css generator options should be normalized to CssModule");

    assert_eq!(parser_options.named_exports, Some(true));
    assert_eq!(generator_options.exports_only, Some(false));
    assert_eq!(generator_options.es_module, Some(true));
  }

  #[test]
  fn reuse_create_data_resource_resolve_data_for_normal_module() {
    let resource_data = ResourceData::new_with_resource("/a.js".to_string());
    let mut create_data = NormalModuleCreateData {
      raw_request: "./a".to_string(),
      request: "/a.js".to_string(),
      user_request: "./a".to_string(),
      resource_resolve_data: NormalModuleCreateDataResource::Owned(resource_data),
      match_resource: None,
      side_effects: None,
      context: None,
    };

    let resource_data = create_data.resource_resolve_data.take_shared();

    assert_eq!(resource_data.resource(), "/a.js");
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
    let global_parser_options = create_global_parser_options_cache(options.module.parser.as_ref());
    let global_generator_options =
      create_global_generator_options_cache(options.module.generator.as_ref());
    Self {
      options,
      global_parser_options,
      global_generator_options,
      resolved_module_options: FxDashMap::default(),
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

    let importer = data.issuer_identifier;
    let raw_request = data.request.clone();

    let mut file_dependencies: ArcResolverPathSet = Default::default();
    let mut missing_dependencies: ArcResolverPathSet = Default::default();

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
        if let Ok((resource, full_matched)) = match_resource(request_without_match_resource) {
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
          return Err(EmptyDependency::new(dependency.range()).into());
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
                .options
                .__references
                .get(ident)
                .map(|object| object.to_string())
            }),
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
      let mut resource_data = ResourceData::new_with_resource(resource);
      plugin_driver
        .normal_module_factory_hooks
        .resolve_for_scheme
        .call(data, &mut resource_data, &scheme)
        .await?;
      resource_data
    } else if !context_scheme.is_none()
      // resource within scheme
      && let Some(resource_data) = {
        let mut resource_data = ResourceData::new_with_resource(resource.clone());
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
        ResourceData::new_with_resource(resource.clone())
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
        file_dependencies = resolve_dependencies.file_dependencies;
        missing_dependencies = resolve_dependencies.missing_dependencies;

        match resource_data {
          Ok(ResolveResult::Resource(resource)) => resource.into(),
          Ok(ResolveResult::Ignored) => {
            let ident = format!("{}/{}", &data.context, resource);
            let module_identifier = ModuleIdentifier::from(format!("ignored|{ident}"));

            let mut raw_module = if matches!(
              dependency_type,
              DependencyType::CssUrl | DependencyType::NewUrl
            ) {
              // use RawModule instead of RawDataUrlModule
              RawModule::new(
                r#"/* (ignored-asset) */
module.exports = "data:,";
"#
                .to_owned(),
                module_identifier,
                format!("{} (ignored-asset)", data.request),
                RuntimeGlobals::MODULE,
              )
              .boxed()
            } else {
              RawModule::new(
                "/* (ignored) */".to_owned(),
                module_identifier,
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
          Err(err) => {
            data.file_dependencies = file_dependencies.into_iter().map(Into::into).collect();
            data.missing_dependencies = missing_dependencies.into_iter().map(Into::into).collect();
            return Err(err);
          }
        }
      }
    };

    let matched_module_rules = if let Some(match_resource_data) = &mut match_resource_data
      && let Ok((module, module_type)) = match_ext(match_resource_data.resource())
    {
      match_module_type = Some(module_type.into());
      match_resource_data.set_resource(module.into());

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
          data.dependencies[0].as_ref(),
          data.issuer.as_deref(),
          data.issuer_layer.as_deref(),
        )
        .await?
    };
    let mut resolved_inline_loaders = vec![];
    for l in inline_loaders {
      resolved_inline_loaders
        .push(resolve_each(plugin_driver, &data.context, &loader_resolver, &l).await?)
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

    let loaders: Vec<BoxLoader> = {
      let mut pre_loaders: Vec<ModuleRuleUseLoader> = vec![];
      let mut post_loaders: Vec<ModuleRuleUseLoader> = vec![];
      let mut normal_loaders: Vec<ModuleRuleUseLoader> = vec![];

      for rule in &matched_module_rules {
        let rule_use = match &rule.r#use {
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
        pre_loaders.len()
          + post_loaders.len()
          + normal_loaders.len()
          + resolved_inline_loaders.len(),
      );

      for l in post_loaders {
        all_loaders
          .push(resolve_each(plugin_driver, &self.options.context, &loader_resolver, &l).await?)
      }

      let mut resolved_normal_loaders = vec![];
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

      all_loaders
    };

    let request = if !loaders.is_empty() {
      let s = loaders
        .iter()
        .map(|i| i.identifier().as_str())
        .collect::<Vec<_>>()
        .join("!");
      format!("{s}!{}", resource_data.resource())
    } else {
      resource_data.resource().to_owned()
    };

    let resolved_module_type = self.calculate_module_type(match_module_type, &matched_module_rules);
    let resolved_module_layer =
      self.calculate_module_layer(data.issuer_layer.as_ref(), &matched_module_rules);

    let resolved_resolve_options = self.calculate_resolve_options(&matched_module_rules);
    let resolved_options =
      self.resolve_module_options(&resolved_module_type, &matched_module_rules);
    let resolved_side_effects = self.calculate_side_effects(&matched_module_rules);
    let resolved_extract_source_map = self.calculate_extract_source_map(&matched_module_rules);
    let mut resolved_parser_and_generator = self
      .plugin_driver
      .registered_parser_and_generator_builder
      .get(&resolved_module_type)
      .ok_or_else(|| {
        error!(
          "No parser registered for '{}'",
          resolved_module_type.as_str()
        )
      })?(resolved_options.clone());
    self
      .plugin_driver
      .normal_module_factory_hooks
      .parser
      .call(
        &resolved_module_type,
        &mut resolved_parser_and_generator,
        resolved_options.parser_options(),
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
        resource_resolve_data: NormalModuleCreateDataResource::Owned(resource_data),
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
      let resource_resolve_data = create_data.resource_resolve_data.take_shared();
      NormalModule::new(
        create_data.request.clone(),
        create_data.user_request.clone(),
        create_data.raw_request.clone(),
        resolved_module_type,
        resolved_module_layer,
        resolved_parser_and_generator,
        resolved_options,
        match_resource_data,
        resource_resolve_data,
        resolved_resolve_options,
        loaders,
        create_data.context.clone().map(|x| x.into()),
        resolved_extract_source_map,
      )
      .boxed()
    };

    self
      .plugin_driver
      .normal_module_factory_hooks
      .module
      .call(data, &create_data, &mut module)
      .await?;

    data.file_dependencies = file_dependencies.into_iter().map(Into::into).collect();
    data.missing_dependencies = missing_dependencies.into_iter().map(Into::into).collect();

    Ok(Some(ModuleFactoryResult::new_with_module(module)))
  }

  async fn calculate_module_rules<'a>(
    &'a self,
    resource_data: &ResourceData,
    dependency: &dyn Dependency,
    issuer: Option<&'a str>,
    issuer_layer: Option<&'a str>,
  ) -> Result<Vec<&'a ModuleRuleEffect>> {
    let mut rules = Vec::new();
    module_rules_matcher(
      &self.options.module.rules,
      resource_data,
      issuer,
      issuer_layer,
      dependency.category(),
      dependency.get_attributes(),
      &mut rules,
    )
    .await?;
    Ok(rules)
  }

  fn calculate_resolve_options(&self, module_rules: &[&ModuleRuleEffect]) -> Option<Arc<Resolve>> {
    let mut resolved: Option<Resolve> = None;
    for rule in module_rules {
      if let Some(rule_resolve) = &rule.resolve {
        if let Some(r) = resolved {
          resolved = Some(r.merge(rule_resolve.to_owned()));
        } else {
          resolved = Some(rule_resolve.to_owned());
        }
      }
    }
    resolved.map(Arc::new)
  }

  fn calculate_side_effects(&self, module_rules: &[&ModuleRuleEffect]) -> Option<bool> {
    let mut side_effect_res = None;
    // side_effects from module rule has higher priority
    for rule in module_rules {
      if rule.side_effects.is_some() {
        side_effect_res = rule.side_effects;
      }
    }
    side_effect_res
  }

  fn calculate_extract_source_map(&self, module_rules: &[&ModuleRuleEffect]) -> Option<bool> {
    let mut extract_source_map_res = None;
    // extract_source_map from module rule has higher priority
    for rule in module_rules {
      if rule.extract_source_map.is_some() {
        extract_source_map_res = rule.extract_source_map;
      }
    }
    extract_source_map_res
  }

  fn resolve_module_options(
    &self,
    module_type: &ModuleType,
    module_rules: &[&ModuleRuleEffect],
  ) -> Arc<ResolvedModuleOptions> {
    let global_parser = self.global_parser_options.get(module_type.as_str());
    let global_generator = self.global_generator_options.get(module_type.as_str());
    resolve_module_options(
      &self.resolved_module_options,
      module_rules,
      module_type,
      global_parser,
      global_generator,
    )
  }

  fn calculate_module_type(
    &self,
    matched_module_type: Option<ModuleType>,
    module_rules: &[&ModuleRuleEffect],
  ) -> ModuleType {
    let mut resolved_module_type = matched_module_type.unwrap_or(ModuleType::JsAuto);
    for module_rule in module_rules {
      if let Some(module_type) = module_rule.r#type {
        resolved_module_type = module_type;
      };
    }

    resolved_module_type
  }

  fn calculate_module_layer(
    &self,
    issuer_layer: Option<&ModuleLayer>,
    module_rules: &[&ModuleRuleEffect],
  ) -> Option<ModuleLayer> {
    let mut resolved_module_layer = issuer_layer;
    for module_rule in module_rules {
      if let Some(module_layer) = &module_rule.layer {
        resolved_module_layer = Some(module_layer);
      };
    }

    resolved_module_layer.cloned()
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
      if let NormalModuleFactoryResolveResult::Module(result) = result {
        return Ok(ModuleFactoryResult::new_with_module(result));
      } else {
        let ident = format!("{}/{}", &data.context, data.request);
        let module_identifier = ModuleIdentifier::from(format!("ignored|{ident}"));

        let mut raw_module = RawModule::new(
          "/* (ignored) */".to_owned(),
          module_identifier,
          format!("{} (ignored)", data.request),
          Default::default(),
        )
        .boxed();

        raw_module.set_factory_meta(FactoryMeta {
          side_effect_free: Some(true),
        });

        return Ok(ModuleFactoryResult::new_with_module(raw_module));
      }
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
) -> Result<Arc<dyn Loader<RunnerContext>>> {
  plugin_driver
    .normal_module_factory_hooks
    .resolve_loader
    .call(context, loader_resolver, l)
    .await?
    .ok_or_else(|| error!("Unable to resolve loader {}", l.loader))
}

#[derive(Debug)]
pub struct NormalModuleCreateData {
  pub raw_request: String,
  pub request: String,
  pub user_request: String,
  pub resource_resolve_data: NormalModuleCreateDataResource,
  pub match_resource: Option<String>,
  pub side_effects: Option<bool>,
  pub context: Option<String>,
}

fn split_element(mut input: &str) -> Vec<&str> {
  use winnow::{
    combinator::separated,
    error::ContextError,
    token::{take_till, take_while},
  };

  separated::<_, _, _, _, ContextError, _, _>(.., take_till(.., '!'), take_while(1.., '!'))
    .parse_next(&mut input)
    .expect("split should never fail")
}

fn match_resource(mut input: &str) -> winnow::ModalResult<(&str, &str)> {
  use winnow::{combinator::terminated, token::take_until};

  let backup = input;

  let res = terminated(take_until(1.., '!'), "!=!").parse_next(&mut input)?;
  let whole_matched = &backup[..backup.len() - input.len()];
  Ok((res, whole_matched))
}

fn match_ext(mut input: &str) -> winnow::ModalResult<(&str, &str)> {
  use winnow::{
    combinator::{alt, delimited, eof, preceded, terminated},
    token::take_until,
  };

  let parser = (
    alt((take_until(0.., ".rspack"), take_until(0.., ".webpack"))),
    preceded(
      alt((".rspack", ".webpack")),
      delimited('[', take_until(1.., ']'), ']'),
    ),
  );

  terminated(parser, eof).parse_next(&mut input)
}

#[test]
fn test_split_element() {
  assert_eq!(split_element("a!a"), vec!["a", "a"]);
  assert_eq!(split_element("a!!a"), vec!["a", "a"]);
  assert_eq!(split_element("!!a!!a!!"), vec!["", "a", "a", ""]);
}

#[test]
fn test_match_ext() {
  assert!(match_ext("foo.webpack[type/javascript]").is_ok());
  let cap = match_ext("foo.webpack[type/javascript]").unwrap();

  assert_eq!(cap, ("foo", "type/javascript"));

  assert_eq!(
    match_ext("foo.css.webpack[javascript/auto]"),
    Ok(("foo.css", "javascript/auto"))
  );

  // Test .rspack support
  assert!(match_ext("foo.rspack[type/javascript]").is_ok());
  let cap = match_ext("foo.rspack[type/javascript]").unwrap();

  assert_eq!(cap, ("foo", "type/javascript"));

  assert_eq!(
    match_ext("foo.css.rspack[javascript/auto]"),
    Ok(("foo.css", "javascript/auto"))
  );
}

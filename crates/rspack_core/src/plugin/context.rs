use std::sync::Arc;

use rspack_cacheable::cacheable;
pub use rspack_util::ArcComputed;
use rspack_util::fx_hash::FxDashMap;

use crate::{
  AssetGeneratorOptions, AssetParserOptions, AssetResourceGeneratorOptions, CompilationHooks,
  CompilerHooks, CompilerOptions, ConcatenatedModuleHooks, ContextModuleFactoryHooks,
  CssAutoOrModuleParserOptions, CssModuleGeneratorOptions, GeneratorOptions, ImportMeta,
  JavascriptParserOptions, JsonGeneratorOptions, JsonParserOptions, MODULE_RULE_ID_UNASSIGNED,
  ModuleRuleEffect, ModuleRuleIds, ModuleType, NormalModuleFactoryHooks, NormalModuleHooks,
  ParserAndGenerator, ParserOptions,
};

pub type BoxedParserAndGenerator = Box<dyn ParserAndGenerator>;
pub type BoxedParserAndGeneratorBuilder =
  Box<dyn 'static + Send + Sync + Fn(Arc<ResolvedModuleOptions>) -> BoxedParserAndGenerator>;

impl<'a> From<&'a ResolvedModuleOptions> for &'a AssetParserOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .parser_options()
      .and_then(ParserOptions::get_asset)
      .expect("should have AssetParserOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a CssAutoOrModuleParserOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .parser_options()
      .and_then(ParserOptions::get_css_module)
      .expect("should have CssAutoOrModuleParserOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a JsonParserOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .parser_options()
      .and_then(ParserOptions::get_json)
      .expect("should have JsonParserOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a JavascriptParserOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .parser_options()
      .and_then(ParserOptions::get_javascript)
      .expect("should have JavascriptParserOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a ImportMeta {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    let javascript_options: &JavascriptParserOptions = owner.into();
    javascript_options.import_meta()
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a AssetGeneratorOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .generator_options()
      .and_then(GeneratorOptions::get_asset)
      .expect("should have AssetGeneratorOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a AssetResourceGeneratorOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .generator_options()
      .and_then(GeneratorOptions::get_asset_resource)
      .expect("should have AssetResourceGeneratorOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a CssModuleGeneratorOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .generator_options()
      .and_then(GeneratorOptions::get_css_module)
      .expect("should have CssModuleGeneratorOptions")
  }
}

impl<'a> From<&'a ResolvedModuleOptions> for &'a JsonGeneratorOptions {
  fn from(owner: &'a ResolvedModuleOptions) -> Self {
    owner
      .generator_options()
      .and_then(GeneratorOptions::get_json)
      .expect("should have JsonGeneratorOptions")
  }
}

#[cacheable]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResolvedModuleOptionsCacheKey {
  pub rule_ids: ModuleRuleIds,
  pub module_type: ModuleType,
}

impl ResolvedModuleOptionsCacheKey {
  pub fn new(module_rules: &[&ModuleRuleEffect], module_type: ModuleType) -> Self {
    let rule_ids = module_rules
      .iter()
      .map(|rule| {
        debug_assert_ne!(
          rule.id, MODULE_RULE_ID_UNASSIGNED,
          "module rule id has not been assigned"
        );
        rule.id
      })
      .collect::<ModuleRuleIds>();

    Self {
      rule_ids,
      module_type,
    }
  }
}

#[cacheable]
#[derive(Debug)]
pub struct ResolvedModuleOptions {
  cache_key: ResolvedModuleOptionsCacheKey,
  parser: Option<ParserOptions>,
  generator: Option<GeneratorOptions>,
}

impl ResolvedModuleOptions {
  pub fn new(
    cache_key: ResolvedModuleOptionsCacheKey,
    parser: Option<ParserOptions>,
    generator: Option<GeneratorOptions>,
  ) -> Self {
    Self {
      cache_key,
      parser,
      generator,
    }
  }

  pub fn cache_key(&self) -> &ResolvedModuleOptionsCacheKey {
    &self.cache_key
  }

  pub fn parser_options_computed<U>(
    self: &Arc<Self>,
    compute: impl FnOnce(&ParserOptions) -> Option<&U>,
  ) -> Option<ArcComputed<Self, U>> {
    ArcComputed::try_new(Arc::clone(self), |owner| {
      owner.parser_options().and_then(compute)
    })
  }

  pub fn generator_options_computed<U>(
    self: &Arc<Self>,
    compute: impl FnOnce(&GeneratorOptions) -> Option<&U>,
  ) -> Option<ArcComputed<Self, U>> {
    ArcComputed::try_new(Arc::clone(self), |owner| {
      owner.generator_options().and_then(compute)
    })
  }

  pub fn parser_options(&self) -> Option<&ParserOptions> {
    self.parser.as_ref()
  }

  pub fn generator_options(&self) -> Option<&GeneratorOptions> {
    self.generator.as_ref()
  }
}

#[non_exhaustive]
pub struct ApplyContext<'c> {
  pub(crate) registered_parser_and_generator_builder:
    &'c mut FxDashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
  pub compiler_hooks: &'c mut CompilerHooks,
  pub compilation_hooks: &'c mut CompilationHooks,
  pub normal_module_factory_hooks: &'c mut NormalModuleFactoryHooks,
  pub context_module_factory_hooks: &'c mut ContextModuleFactoryHooks,
  pub normal_module_hooks: &'c mut NormalModuleHooks,
  pub concatenated_module_hooks: &'c mut ConcatenatedModuleHooks,

  pub compiler_options: &'c CompilerOptions,
}

impl ApplyContext<'_> {
  pub fn register_parser_and_generator_builder(
    &mut self,
    module_type: ModuleType,
    parser_and_generator_builder: BoxedParserAndGeneratorBuilder,
  ) {
    self
      .registered_parser_and_generator_builder
      .insert(module_type, parser_and_generator_builder);
  }
}

use std::{ops::Deref, sync::Arc};

use rspack_cacheable::{cacheable, with::AsInnerConverter};
use rspack_util::fx_hash::FxDashMap;

use crate::{
  AssetGeneratorOptions, AssetParserOptions, AssetResourceGeneratorOptions, CompilationHooks,
  CompilerHooks, CompilerOptions, ConcatenatedModuleHooks, ContextModuleFactoryHooks,
  CssAutoOrModuleParserOptions, CssModuleGeneratorOptions, GeneratorOptions, JsonGeneratorOptions,
  JsonParserOptions, MODULE_RULE_ID_UNASSIGNED, ModuleRuleEffect, ModuleRuleIds, ModuleType,
  NormalModuleFactoryHooks, NormalModuleHooks, ParserAndGenerator, ParserOptions,
};

pub type BoxedParserAndGenerator = Box<dyn ParserAndGenerator>;
pub type BoxedParserAndGeneratorBuilder =
  Box<dyn 'static + Send + Sync + Fn(Arc<ResolvedModuleOptions>) -> BoxedParserAndGenerator>;

#[derive(Debug, Clone)]
pub struct ArcComputed<T, U> {
  owner: Arc<T>,
  computed: *const U,
}

impl<T, U> ArcComputed<T, U> {
  pub fn new(owner: Arc<T>, compute: impl FnOnce(&T) -> &U) -> Self {
    let computed = compute(&owner) as *const U;
    Self { owner, computed }
  }

  pub fn try_new(owner: Arc<T>, compute: impl FnOnce(&T) -> Option<&U>) -> Option<Self> {
    let computed = compute(&owner)? as *const U;
    Some(Self { owner, computed })
  }

  pub fn owner(&self) -> &Arc<T> {
    &self.owner
  }
}

impl<T, U> AsInnerConverter for ArcComputed<T, U>
where
  for<'a> &'a U: From<&'a T>,
{
  type Inner = Arc<T>;

  fn to_inner(&self) -> &Self::Inner {
    &self.owner
  }

  fn from_inner(data: Self::Inner) -> Self {
    Self::new(data, |owner| owner.into())
  }
}

impl<T, U> Deref for ArcComputed<T, U> {
  type Target = U;

  fn deref(&self) -> &Self::Target {
    // SAFETY: `computed` is created from a shared reference into `owner`.
    // `owner` is kept alive by this struct and the option objects are immutable.
    unsafe { &*self.computed }
  }
}

impl<T, U> AsRef<U> for ArcComputed<T, U> {
  fn as_ref(&self) -> &U {
    self
  }
}

unsafe impl<T, U> Send for ArcComputed<T, U>
where
  Arc<T>: Send,
  U: Sync,
{
}

unsafe impl<T, U> Sync for ArcComputed<T, U>
where
  Arc<T>: Sync,
  U: Sync,
{
}

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

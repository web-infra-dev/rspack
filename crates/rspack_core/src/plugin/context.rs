use rspack_util::fx_hash::FxDashMap;

use crate::{
  CompilationHooks, CompilerHooks, ContextModuleFactoryHooks, GeneratorOptions, ModuleType,
  NormalModuleFactoryHooks, NormalModuleHooks, ParserAndGenerator, ParserOptions,
};

#[derive(Debug, Default)]
pub struct PluginContext<T = ()> {
  pub context: T,
}

impl PluginContext {
  pub fn new() -> Self {
    Self::with_context(())
  }
}

impl<T> PluginContext<T> {
  pub fn with_context(context: T) -> Self {
    Self { context }
  }

  pub fn into_context(self) -> T {
    self.context
  }
}

// pub type BoxedParser = Box<dyn Parser>;
pub type BoxedParserAndGenerator = Box<dyn ParserAndGenerator>;
pub type BoxedParserAndGeneratorBuilder = Box<
  dyn 'static
    + Send
    + Sync
    + Fn(Option<&ParserOptions>, Option<&GeneratorOptions>) -> BoxedParserAndGenerator,
>;

pub struct ApplyContext<'c> {
  pub(crate) registered_parser_and_generator_builder:
    &'c mut FxDashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
  pub compiler_hooks: &'c mut CompilerHooks,
  pub compilation_hooks: &'c mut CompilationHooks,
  pub normal_module_factory_hooks: &'c mut NormalModuleFactoryHooks,
  pub context_module_factory_hooks: &'c mut ContextModuleFactoryHooks,
  pub normal_module_hooks: &'c mut NormalModuleHooks,
}

impl<'c> ApplyContext<'c> {
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

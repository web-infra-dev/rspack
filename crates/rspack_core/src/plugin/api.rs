use std::fmt::Debug;

use rspack_error::Result;
use rspack_sources::BoxSource;
use rspack_util::fx_hash::FxDashMap;

use crate::{
  AssetInfo, CompilationHooks, CompilerHooks, CompilerOptions, ContextModuleFactoryHooks,
  GeneratorOptions, ModuleType, NormalModuleFactoryHooks, NormalModuleHooks, ParserAndGenerator,
  ParserOptions, PluginContext,
};

#[async_trait::async_trait]
pub trait Plugin: Debug + Send + Sync {
  fn name(&self) -> &'static str {
    "unknown"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    Ok(())
  }
}

pub type BoxPlugin = Box<dyn Plugin>;

pub trait PluginExt {
  fn boxed(self) -> BoxPlugin;
}

impl<T: Plugin + 'static> PluginExt for T {
  fn boxed(self) -> BoxPlugin {
    Box::new(self)
  }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum AssetContent {
//   Buffer(Vec<u8>),
//   String(String),
// }

#[derive(Debug, Clone)]
pub struct RenderManifestEntry {
  pub source: BoxSource,
  filename: String,
  pub info: AssetInfo,
  // pub identifier: String,
  // hash?: string;
  pub(crate) auxiliary: bool,
  has_filename: bool, /* webpack only asset has filename, js/css/wasm has filename template */
}

impl RenderManifestEntry {
  pub fn new(
    source: BoxSource,
    filename: String,
    info: AssetInfo,
    auxiliary: bool,
    has_filename: bool,
  ) -> Self {
    Self {
      source,
      filename,
      info,
      auxiliary,
      has_filename,
    }
  }

  pub fn source(&self) -> &BoxSource {
    &self.source
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }

  pub fn has_filename(&self) -> bool {
    self.has_filename
  }
}

// pub trait Parser: Debug + Sync + Send {
//   fn parse(
//     &self,
//     module_type: ModuleType,
//     args: ParseModuleArgs,
//   ) -> Result<TWithDiagnosticArray<BoxModule>>;
// }

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

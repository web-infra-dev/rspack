use std::time::{Duration, Instant};

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, CompilerOptions, ModuleType, NormalModuleFactoryParser, ParserAndGenerator,
  ParserOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};

use crate::parser_plugin::RslibParserPlugin;

#[derive(Debug)]
pub struct RslibPluginOptions {
  pub intercept_api_plugin: bool,
}

#[derive(Debug)]
pub struct ProgressPluginStateInfo {
  pub value: String,
  pub time: Instant,
  pub duration: Option<Duration>,
}

#[plugin]
#[derive(Debug)]
pub struct RslibPlugin {
  options: RslibPluginOptions,
}

impl RslibPlugin {
  pub fn new(options: RslibPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(NormalModuleFactoryParser for RslibPlugin)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut dyn ParserAndGenerator,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(
      Box::new(RslibParserPlugin::new(self.options.intercept_api_plugin))
        as BoxJavascriptParserPlugin,
    );
  }

  Ok(())
}

#[async_trait]
impl Plugin for RslibPlugin {
  fn name(&self) -> &'static str {
    "rslib"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));

    Ok(())
  }
}

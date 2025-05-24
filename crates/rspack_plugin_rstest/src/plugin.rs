use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, Compilation, CompilationParams, CompilerCompilation, CompilerOptions, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  parser_and_generator::JavaScriptParserAndGenerator, BoxJavascriptParserPlugin,
};

use crate::{
  module_path_name_dependency::ModulePathNameDependencyTemplate, parser_plugin::RstestParserPlugin,
};

#[derive(Debug)]
pub struct RstestPluginOptions {
  pub module_path_name: bool,
}

#[derive(Debug)]
pub struct ProgressPluginStateInfo {
  pub value: String,
  pub time: Instant,
  pub duration: Option<Duration>,
}

#[plugin]
#[derive(Debug)]
pub struct RstestPlugin {
  options: RstestPluginOptions,
}

impl RstestPlugin {
  pub fn new(options: RstestPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(NormalModuleFactoryParser for RstestPlugin)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut dyn ParserAndGenerator,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if module_type.is_js_like()
    && let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>()
  {
    parser.add_parser_plugin(Box::<RstestParserPlugin>::default() as BoxJavascriptParserPlugin);
  }

  Ok(())
}

#[plugin_hook(CompilerCompilation for RstestPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_template(
    ModulePathNameDependencyTemplate::template_type(),
    Arc::new(ModulePathNameDependencyTemplate::default()),
  );
  Ok(())
}

#[async_trait]
impl Plugin for RstestPlugin {
  fn name(&self) -> &'static str {
    "rstest"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    if self.options.module_path_name {
      ctx
        .context
        .compiler_hooks
        .compilation
        .tap(compilation::new(self));

      ctx
        .context
        .normal_module_factory_hooks
        .parser
        .tap(nmf_parser::new(self));
    }

    Ok(())
  }
}

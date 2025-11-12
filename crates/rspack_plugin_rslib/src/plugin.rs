use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use rspack_core::{
  Compilation, CompilationParams, CompilerCompilation, CompilerFinishMake, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_asset::AssetParserAndGenerator;
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};

use crate::{
  asset::RslibAssetParserAndGenerator, import_dependency::RslibDependencyTemplate,
  import_external::replace_import_dependencies_for_external_modules,
  parser_plugin::RslibParserPlugin,
};

#[derive(Debug)]
pub struct RslibPluginOptions {
  pub intercept_api_plugin: bool,
  pub force_node_shims: bool,
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
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>() {
    if module_type.is_js_like() {
      parser.add_parser_plugin(
        Box::new(RslibParserPlugin::new(self.options.intercept_api_plugin))
          as BoxJavascriptParserPlugin,
      );
    }

    if module_type.is_js_esm() && self.options.force_node_shims {
      parser.add_parser_plugin(Box::new(
        rspack_plugin_javascript::node_stuff_plugin::NodeStuffPlugin,
      ) as BoxJavascriptParserPlugin);
    }
  } else if parser.is::<AssetParserAndGenerator>() {
    // Already added RslibParserPlugin, do nothing
    *parser = Box::new(RslibAssetParserAndGenerator(
      parser
        .downcast_ref::<AssetParserAndGenerator>()
        .expect("is AssetParser")
        .clone(),
    ))
  }

  Ok(())
}

#[plugin_hook(CompilerCompilation for RslibPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_template(
    RslibDependencyTemplate::template_type(),
    Arc::new(RslibDependencyTemplate::default()),
  );
  Ok(())
}

#[plugin_hook(CompilerFinishMake for RslibPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  // Replace ImportDependency instances with RslibImportDependency for external modules
  replace_import_dependencies_for_external_modules(compilation)?;
  Ok(())
}

impl Plugin for RslibPlugin {
  fn name(&self) -> &'static str {
    "rslib"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    Ok(())
  }
}

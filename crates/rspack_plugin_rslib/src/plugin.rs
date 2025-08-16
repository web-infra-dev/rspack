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
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, parser_and_generator::JavaScriptParserAndGenerator,
};
use rspack_plugin_library::{
  ModernModuleImportDependencyTemplate, replace_import_dependencies_for_external_modules,
};

use crate::parser_plugin::RslibParserPlugin;

#[derive(Debug)]
pub struct RslibPluginOptions {
  pub intercept_api_plugin: bool,
  pub compact_external_module_dynamic_import: bool,
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

#[plugin_hook(CompilerCompilation for RslibPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_template(
    ModernModuleImportDependencyTemplate::template_type(),
    Arc::new(ModernModuleImportDependencyTemplate::default()),
  );
  Ok(())
}

#[plugin_hook(CompilerFinishMake for RslibPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  // Replace ImportDependency instances with ModernModuleImportDependency for external modules
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

    if self.options.compact_external_module_dynamic_import {
      ctx.compiler_hooks.finish_make.tap(finish_make::new(self));
    }

    Ok(())
  }
}

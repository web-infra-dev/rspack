use std::path::Path;

pub use rspack_swc::swc_ecma_ast as ast;

use tracing::instrument;

use crate::{
  plugin::PluginTransformAstHookOutput, plugin_driver::PluginDriver, Loader,
  PluginTransformHookOutput,
};

#[instrument(skip(ast, plugin_driver))]
#[inline]
pub fn transform_ast(
  path: &str,
  ast: ast::Module,
  plugin_driver: &PluginDriver,
) -> PluginTransformAstHookOutput {
  plugin_driver.transform_ast(path, ast)
}

#[instrument(skip_all)]
#[inline]
pub fn transform(
  uri: &str,
  loader: &mut Option<Loader>,
  source: String,
  plugin_driver: &PluginDriver,
) -> PluginTransformHookOutput {
  plugin_driver.transform(uri, loader, source)
}

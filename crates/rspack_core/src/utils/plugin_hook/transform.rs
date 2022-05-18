pub use rspack_swc::swc_ecma_ast as ast;
use std::path::Path;
use tracing::instrument;

use crate::{plugin::PluginTransformHookOutput, plugin_driver::PluginDriver};

#[instrument(skip(ast, plugin_driver))]
#[inline]
pub fn transform_ast(
  path: &Path,
  ast: ast::Module,
  plugin_driver: &PluginDriver,
) -> PluginTransformHookOutput {
  plugin_driver.transform_ast(path, ast)
}

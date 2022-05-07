use std::path::Path;

use tracing::instrument;

use crate::{plugin::PluginTransformHookOutput, plugin_driver::PluginDriver};

#[instrument(skip(ast, plugin_driver))]
#[inline]
pub fn transform(
  path: &Path,
  ast: ast::Module,
  plugin_driver: &PluginDriver,
) -> PluginTransformHookOutput {
  plugin_driver.transform(path, ast)
}

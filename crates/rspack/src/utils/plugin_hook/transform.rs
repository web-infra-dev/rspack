use tracing::instrument;

use crate::{plugin_driver::PluginDriver, traits::plugin::TransformHookOutput};

#[instrument(skip(ast, plugin_driver))]
#[inline]
pub fn transform(ast: swc_ecma_ast::Program, plugin_driver: &PluginDriver) -> TransformHookOutput {
  let plugin_result = plugin_driver.transform(ast);

  plugin_result
}

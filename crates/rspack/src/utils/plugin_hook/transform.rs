use tracing::instrument;

use crate::{plugin_driver::PluginDriver, traits::plugin::TransformHookOutput};

#[instrument(skip(ast, plugin_driver))]
#[inline]
pub fn transform(ast: swc_ecma_ast::Module, plugin_driver: &PluginDriver) -> TransformHookOutput {
  plugin_driver.transform(ast)
}

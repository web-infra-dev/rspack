use tracing::instrument;

use crate::{plugin_driver::PluginDriver, traits::plugin::TransformHookOutput};

#[instrument]
#[inline]
pub fn transform(ast: swc_ecma_ast::Program, plugin_dirver: &PluginDriver) -> TransformHookOutput {
  let plugin_result = plugin_dirver.transform(ast);

  plugin_result
}

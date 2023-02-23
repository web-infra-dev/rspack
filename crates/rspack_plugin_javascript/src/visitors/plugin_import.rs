use either::Either;
use swc_core::ecma::{transforms::base::pass::noop, visit::Fold};
use swc_plugin_import::PluginImportConfig;

pub fn plugin_import(config: Option<&Vec<PluginImportConfig>>) -> impl Fold + '_ {
  if let Some(config) = config {
    Either::Left(swc_plugin_import::plugin_import(config))
  } else {
    Either::Right(noop())
  }
}

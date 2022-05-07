use std::collections::HashMap;

use rspack_core::{BundleOptions, Plugin};
use rspack_plugin_css::plugin::CssSourcePlugin;

pub mod log;

pub fn inject_built_in_plugins(
  mut user_plugins: Vec<Box<dyn Plugin>>,
  options: &mut BundleOptions,
) -> Vec<Box<dyn Plugin>> {
  let mut plugins: Vec<Box<dyn Plugin>> = vec![Box::new(rspack_plugin_react::ReactPlugin {
    runtime: options.react.runtime,
  })];
  if let Some(loader_options) = options.loader.take() {
    plugins.push(Box::new(rspack_plugin_loader::LoaderPlugin {
      options: loader_options,
    }));
  }
  plugins.append(&mut user_plugins);
  let css_plugin: Box<CssSourcePlugin> = std::default::Default::default();
  plugins.push(css_plugin);
  plugins
}

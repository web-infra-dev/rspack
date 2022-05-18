use rspack_core::{BundleOptions, NormalizedBundleOptions, Plugin};
use rspack_plugin_css::plugin::CssSourcePlugin;

pub mod log;
pub mod rayon;

pub fn inject_built_in_plugins(
  mut user_plugins: Vec<Box<dyn Plugin>>,
  options: &NormalizedBundleOptions,
) -> Vec<Box<dyn Plugin>> {
  let mut plugins: Vec<Box<dyn Plugin>> = vec![Box::new(rspack_plugin_react::ReactPlugin {
    runtime: options.react.runtime,
  })];
  plugins.push(Box::new(rspack_plugin_loader::LoaderInterpreterPlugin));
  // start --- injected user plugins
  plugins.push(Box::new(rspack_plugin_progress::ProgressPlugin::new()));
  plugins.append(&mut user_plugins);
  // end --- injected user plugins
  if options.inline_style {
    plugins.push(Box::new(rspack_plugin_style::StyleLoaderPlugin {}));
  } else {
    let css_plugin: Box<CssSourcePlugin> = std::default::Default::default();
    plugins.push(css_plugin);
  }
  plugins.push(Box::new(
    rspack_plugin_mock_buitins::MockBuitinsPlugin::new(),
  ));
  plugins
}

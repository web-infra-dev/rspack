use rspack_core::Plugin;
use rspack_plugin_css::plugin::CssSourcePlugin;

pub mod log;

pub fn inject_built_in_plugins(mut plugins: Vec<Box<dyn Plugin>>) -> Vec<Box<dyn Plugin>> {
  let css_plugin: Box<CssSourcePlugin> = std::default::Default::default();
  plugins.push(css_plugin);
  plugins
}

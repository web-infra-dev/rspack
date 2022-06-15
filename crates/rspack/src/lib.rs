pub use rspack_core::Compiler;
use rspack_core::{CompilerOptions, Plugin};
pub fn rspack(options: CompilerOptions, mut plugins: Vec<Box<dyn Plugin>>) -> Compiler {
  plugins.push(Box::new(rspack_plugin_javascript::JsPlugin {}));
  plugins.push(Box::new(rspack_plugin_css::CssPlugin::default()));
  Compiler::new(options, plugins)
}

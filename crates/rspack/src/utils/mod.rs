use rspack_core::{BundleOptions, NormalizedBundleOptions, Plugin};
use rspack_plugin_stylesource::plugin::StyleSourcePlugin;

pub mod log;
pub mod rayon;

pub fn inject_built_in_plugins(
  mut user_plugins: Vec<Box<dyn Plugin>>,
  options: &NormalizedBundleOptions,
) -> Vec<Box<dyn Plugin>> {
  let mut plugins: Vec<Box<dyn Plugin>> = vec![Box::new(rspack_plugin_react::ReactPlugin {
    runtime: options.react.runtime,
  })];
  // start --- injected user plugins
  plugins.push(Box::new(rspack_plugin_progress::ProgressPlugin::new()));
  let use_svgr = options
    .loader
    .get("svg")
    .map(|x| matches!(x, &Loader::Svgr))
    .unwrap_or(false);
  if use_svgr {
    plugins.push(Box::new(rspack_plugin_svgr::SvgrPlugin {}))
  }
  plugins.append(&mut user_plugins);
  // end --- injected user plugins
  plugins.push(Box::new(rspack_plugin_loader::LoaderInterpreterPlugin));
  if options.inline_style {
    // todo fix 后续是否 集成 进 style_source
    // 方便我在 hmr 的时候 切割节点
    plugins.push(Box::new(rspack_plugin_style::StyleLoaderPlugin {}));
  } else {
    // 处理所有样式
    let stylesource_plugin: Box<StyleSourcePlugin> = std::default::Default::default();
    plugins.push(stylesource_plugin);
  }
  plugins.push(Box::new(
    rspack_plugin_mock_buitins::MockBuitinsPlugin::new(),
  ));
  plugins
}

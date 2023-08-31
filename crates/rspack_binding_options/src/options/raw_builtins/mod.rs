mod raw_banner;
mod raw_copy;
mod raw_html;
mod raw_progress;
mod raw_swc_js_minimizer;
mod raw_to_be_deprecated;

use napi::{
  bindgen_prelude::{FromNapiValue, ToNapiValue},
  JsUnknown,
};
use napi_derive::napi;
use rspack_core::{
  BoxPlugin, CopyPluginConfig, Define, DefinePlugin, PluginExt, Provide, ProvidePlugin,
};
use rspack_error::Result;
use rspack_napi_shared::NapiResultExt;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_copy::CopyPlugin;
use rspack_plugin_entry::EntryPlugin;
use rspack_plugin_externals::{
  electron_target_plugin, http_externals_plugin, node_target_plugin, ExternalsPlugin,
};
use rspack_plugin_html::HtmlPlugin;
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_swc_css_minimizer::SwcCssMinimizerPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerPlugin;

pub use self::{
  raw_banner::RawBannerConfig, raw_copy::RawCopyConfig, raw_html::RawHtmlPluginConfig,
  raw_progress::RawProgressPluginConfig, raw_swc_js_minimizer::RawMinification,
};
use crate::{
  RawEntryPluginOptions, RawExternalsPluginOptions, RawHttpExternalsPluginOptions, RawOptionsApply,
};

#[napi(string_enum)]
#[derive(Debug)]
pub enum BuiltinPluginKind {
  Define,
  Provide,
  Banner,
  Progress,
  Copy,
  Html,
  SwcJsMinimizer,
  SwcCssMinimizer,
  Entry,
  Externals,
  NodeTarget,
  ElectronTarget,
  HttpExternals,
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub kind: BuiltinPluginKind,
  pub options: JsUnknown,
}

impl RawOptionsApply for BuiltinPlugin {
  type Options = ();

  fn apply(
    self,
    plugins: &mut Vec<BoxPlugin>,
  ) -> std::result::Result<Self::Options, rspack_error::Error> {
    match self.kind {
      BuiltinPluginKind::Define => {
        let plugin = DefinePlugin::new(downcast_into::<Define>(self.options)?).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::Provide => {
        let plugin = ProvidePlugin::new(downcast_into::<Provide>(self.options)?).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::Banner => {
        let plugin =
          BannerPlugin::new(downcast_into::<RawBannerConfig>(self.options)?.try_into()?).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::SwcJsMinimizer => {
        let plugin =
          SwcJsMinimizerPlugin::new(downcast_into::<RawMinification>(self.options)?.try_into()?)
            .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::SwcCssMinimizer => plugins.push(SwcCssMinimizerPlugin {}.boxed()),
      BuiltinPluginKind::Progress => {
        let plugin =
          ProgressPlugin::new(downcast_into::<RawProgressPluginConfig>(self.options)?.into())
            .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::Copy => {
        let plugin = CopyPlugin::new(
          CopyPluginConfig::from(downcast_into::<RawCopyConfig>(self.options)?).patterns,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::Html => {
        let plugin =
          HtmlPlugin::new(downcast_into::<RawHtmlPluginConfig>(self.options)?.into()).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::Entry => {
        let plugin_options = downcast_into::<RawEntryPluginOptions>(self.options)?;
        let context = plugin_options.context.into();
        let entry_request = plugin_options.entry;
        let options = plugin_options.options.into();
        let plugin = EntryPlugin::new(context, entry_request, options).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::Externals => {
        let plugin_options = downcast_into::<RawExternalsPluginOptions>(self.options)?;
        let externals = plugin_options
          .externals
          .into_iter()
          .map(|e| e.try_into())
          .collect::<Result<Vec<_>>>()?;
        let plugin = ExternalsPlugin::new(plugin_options.r#type, externals).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginKind::NodeTarget => plugins.push(node_target_plugin()),
      BuiltinPluginKind::ElectronTarget => {
        let context = downcast_into::<String>(self.options)?;
        electron_target_plugin(context.into(), plugins)
      }
      BuiltinPluginKind::HttpExternals => {
        let plugin_options = downcast_into::<RawHttpExternalsPluginOptions>(self.options)?;
        let plugin = http_externals_plugin(plugin_options.css);
        plugins.push(plugin);
      }
    }
    Ok(())
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  <T as FromNapiValue>::from_unknown(o).into_rspack_result()
}

// TO BE DEPRECATED
pub use raw_to_be_deprecated::RawBuiltins;

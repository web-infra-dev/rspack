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
use rspack_error::{Error, Result};
use rspack_napi_shared::NapiResultExt;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_copy::CopyPlugin;
use rspack_plugin_entry::EntryPlugin;
use rspack_plugin_html::HtmlPlugin;
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_swc_css_minimizer::SwcCssMinimizerPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerPlugin;

pub use self::{
  raw_banner::RawBannerConfig, raw_copy::RawCopyConfig, raw_html::RawHtmlPluginConfig,
  raw_progress::RawProgressPluginConfig, raw_swc_js_minimizer::RawMinification,
};
use crate::RawEntryPluginOptions;

#[napi]
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
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub kind: BuiltinPluginKind,
  pub options: JsUnknown,
}

impl TryFrom<BuiltinPlugin> for BoxPlugin {
  type Error = Error;

  fn try_from(value: BuiltinPlugin) -> Result<Self> {
    let plugin = match value.kind {
      BuiltinPluginKind::Define => {
        DefinePlugin::new(downcast_into::<Define>(value.options)?).boxed()
      }
      BuiltinPluginKind::Provide => {
        ProvidePlugin::new(downcast_into::<Provide>(value.options)?).boxed()
      }
      BuiltinPluginKind::Banner => {
        BannerPlugin::new(downcast_into::<RawBannerConfig>(value.options)?.try_into()?).boxed()
      }
      BuiltinPluginKind::SwcJsMinimizer => {
        SwcJsMinimizerPlugin::new(downcast_into::<RawMinification>(value.options)?.try_into()?)
          .boxed()
      }
      BuiltinPluginKind::SwcCssMinimizer => SwcCssMinimizerPlugin {}.boxed(),
      BuiltinPluginKind::Progress => {
        ProgressPlugin::new(downcast_into::<RawProgressPluginConfig>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::Copy => CopyPlugin::new(
        CopyPluginConfig::from(downcast_into::<RawCopyConfig>(value.options)?).patterns,
      )
      .boxed(),
      BuiltinPluginKind::Html => {
        HtmlPlugin::new(downcast_into::<RawHtmlPluginConfig>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::Entry => {
        let plugin_options = downcast_into::<RawEntryPluginOptions>(value.options)?;
        let context = plugin_options.context.into();
        let entry_request = plugin_options.entry;
        let options = plugin_options.options.into();
        EntryPlugin::new(context, entry_request, options).boxed()
      }
    };
    Ok(plugin)
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  <T as FromNapiValue>::from_unknown(o).into_rspack_result()
}

// TO BE DEPRECATED
pub use raw_to_be_deprecated::RawBuiltins;
